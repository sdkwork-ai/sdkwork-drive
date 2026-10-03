import { useEffect, useRef, useState } from 'react';
import type { KeyboardEvent as ReactKeyboardEvent } from 'react';
import { Maximize2, Minimize2, Pause, Play } from 'lucide-react';
import type { FilePreviewLabels } from '../../i18n/filePreviewLabels';
import { useMediaPreviewUrl, type MediaPreviewSource } from '../../hooks/useMediaPreviewUrl';
import { formatPlaybackTime } from '../../utils/formatPlaybackTime';
import { PreviewErrorPanel, PreviewLoadingPanel } from '../PreviewStatePanels';
import {
  PREVIEW_CANVAS_CLASS,
  PREVIEW_ICON_BUTTON_CLASS,
  PREVIEW_INPUT_CLASS,
  PREVIEW_TOOLBAR_CLASS,
} from '../previewStyles';

export interface VideoFilePreviewProps {
  labels: FilePreviewLabels;
  name: string;
  source: MediaPreviewSource;
}

/** 播放速度档位：覆盖「慢速跟读」到「两倍速略过」，1× 常驻列表。 */
const PLAYBACK_RATES = [0.5, 1, 1.25, 1.5, 2] as const;

/**
 * `timeupdate` 在部分浏览器里能到每秒十几次，节流到约 4 次/秒。
 *
 * 读数只需要跟手，不需要逐帧；每次 setState 都会重渲整个工具栏，不节流会在长视频里
 * 变成持续的渲染压力（拖慢的是视频本身的解码，用户看到的是掉帧）。
 */
const TIME_UPDATE_INTERVAL_MS = 250;

/** 方向键步进秒数。 */
const SEEK_STEP_SECONDS = 5;

/**
 * 快捷键的例外集合。
 *
 * 焦点落在按钮/下拉/滑块上时，空格与方向键属于这些控件自己的语义（空格激活按钮、
 * 下拉切换选项、滑块调整数值），抢过来会同时触发两次或让控件失灵。
 */
const INTERACTIVE_TARGET_SELECTOR =
  'button, input, select, textarea, a[href], [role="button"], [role="slider"], [role="tab"]';

/**
 * 全屏相关的可选能力。
 *
 * 声明成「可选成员」的结构类型而不是对 `HTMLElement` 做断言：旧版 Safari 只有
 * `webkitRequestFullscreen`，而 jsdom 两者都没有；用可选成员 + 运行时探测可以在
 * 不支持全屏的宿主里安静退化成「按钮无效」，而不是抛 TypeError。
 */
interface FullscreenCapableElement {
  requestFullscreen?: (options?: FullscreenOptions) => Promise<void>;
  webkitRequestFullscreen?: () => Promise<void> | void;
}

interface FullscreenCapableDocument {
  fullscreenElement?: Element | null;
  webkitFullscreenElement?: Element | null;
  exitFullscreen?: () => Promise<void>;
  webkitExitFullscreen?: () => Promise<void> | void;
}

interface ActiveKeyboardEvent {
  key: string;
  target: EventTarget | null;
}

/** jsdom 等宿主没有实现 `play()`，自动播放策略也会让它 reject；两种都不该冒泡成未处理拒绝。 */
function startPlayback(element: HTMLMediaElement): void {
  try {
    const pending: unknown = element.play();
    if (pending instanceof Promise) {
      pending.catch(() => undefined);
    }
  } catch {
    // 宿主不支持播放：交给 onError/界面状态反馈，不在渲染路径上抛错。
  }
}

function isFullscreenActive(): boolean {
  const fullscreenDocument: FullscreenCapableDocument = document;
  return Boolean(fullscreenDocument.fullscreenElement ?? fullscreenDocument.webkitFullscreenElement);
}

function requestElementFullscreen(element: FullscreenCapableElement): void {
  const request = element.requestFullscreen ?? element.webkitRequestFullscreen;
  if (typeof request !== 'function') {
    // 不支持全屏（含 jsdom）：按钮保持无操作，而不是报错。
    return;
  }
  try {
    const pending: unknown = request.call(element);
    if (pending instanceof Promise) {
      pending.catch(() => undefined);
    }
  } catch {
    // 非用户手势触发的全屏请求会被浏览器拒绝。
  }
}

function exitElementFullscreen(): void {
  const fullscreenDocument: FullscreenCapableDocument = document;
  const exit = fullscreenDocument.exitFullscreen ?? fullscreenDocument.webkitExitFullscreen;
  if (typeof exit !== 'function') {
    return;
  }
  try {
    const pending: unknown = exit.call(document);
    if (pending instanceof Promise) {
      pending.catch(() => undefined);
    }
  } catch {
    // 退出全屏失败不影响预览本身。
  }
}

/**
 * 视频预览：原生 `<video controls>` + 一层轻量工具栏。
 *
 * 自绘播放器要接管缓冲、字幕、画中画、投屏、移动端手势这些原生已经做好的行为，
 * 收益只是外观统一。这里只补原生控件缺的三件事：倍速、全屏按钮和一处不随控件
 * 显隐跳动的时间读数；键位只在容器获得焦点时生效，不做全局监听——预览面板经常
 * 只是页面里的一个抽屉，全局抢空格会让宿主自己的快捷键失效。
 */
export function VideoFilePreview({ labels, name, source }: VideoFilePreviewProps) {
  const url = useMediaPreviewUrl(source);
  const wrapperRef = useRef<HTMLDivElement | null>(null);
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const lastTimeUpdateRef = useRef(0);
  const [failed, setFailed] = useState(false);
  const [playing, setPlaying] = useState(false);
  const [muted, setMuted] = useState(false);
  const [playbackRate, setPlaybackRate] = useState(1);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [fullscreen, setFullscreen] = useState(false);

  useEffect(() => {
    const handleFullscreenChange = () => setFullscreen(isFullscreenActive());
    document.addEventListener('fullscreenchange', handleFullscreenChange);
    document.addEventListener('webkitfullscreenchange', handleFullscreenChange);
    return () => {
      document.removeEventListener('fullscreenchange', handleFullscreenChange);
      document.removeEventListener('webkitfullscreenchange', handleFullscreenChange);
    };
  }, []);

  // 依赖里带上 url：媒体元素要等 URL 就绪后才挂载，只依赖倍速/静音会在首帧丢掉设置。
  useEffect(() => {
    const video = videoRef.current;
    if (video) {
      video.playbackRate = playbackRate;
    }
  }, [playbackRate, url]);

  useEffect(() => {
    const video = videoRef.current;
    if (video) {
      video.muted = muted;
    }
  }, [muted, url]);

  const togglePlayback = () => {
    const video = videoRef.current;
    if (!video) {
      return;
    }
    if (playing) {
      video.pause();
      setPlaying(false);
      return;
    }
    // 状态先行：原生控件或键盘触发的播放由 onPlay 校正，避免依赖 `paused`
    // 在 jsdom 与部分移动端浏览器里不随 play() 同步更新的差异。
    setPlaying(true);
    startPlayback(video);
  };

  const toggleMute = () => setMuted((value) => !value);

  const toggleFullscreen = () => {
    const wrapper = wrapperRef.current;
    if (!wrapper) {
      return;
    }
    if (isFullscreenActive()) {
      exitElementFullscreen();
      return;
    }
    requestElementFullscreen(wrapper);
  };

  const seekBy = (delta: number) => {
    const video = videoRef.current;
    if (!video) {
      return;
    }
    const limit = Number.isFinite(video.duration) ? video.duration : video.currentTime + delta;
    const next = Math.min(Math.max(video.currentTime + delta, 0), Math.max(limit, 0));
    video.currentTime = next;
    setCurrentTime(next);
  };

  const handleKeyDown = (event: ReactKeyboardEvent<HTMLDivElement>) => {
    const keyboardEvent: ActiveKeyboardEvent = { key: event.key, target: event.target };
    if (keyboardEvent.target instanceof Element
      && keyboardEvent.target.closest(INTERACTIVE_TARGET_SELECTOR) !== null) {
      return;
    }
    /*
     * 播放器自己消费掉的按键必须 stopPropagation：预览层用左右方向键切换相邻文件，
     * 事件冒泡上去就会"快退一下、同时换一个文件"。键盘焦点在播放器里时才拦截，
     * 焦点在别处时方向键仍然归导航——这也是主流播放器的行为。
     */
    switch (keyboardEvent.key) {
      case ' ':
      case 'Spacebar':
        event.preventDefault();
        event.stopPropagation();
        togglePlayback();
        break;
      case 'ArrowLeft':
        event.preventDefault();
        event.stopPropagation();
        seekBy(-SEEK_STEP_SECONDS);
        break;
      case 'ArrowRight':
        event.preventDefault();
        event.stopPropagation();
        seekBy(SEEK_STEP_SECONDS);
        break;
      case 'm':
      case 'M':
        event.preventDefault();
        event.stopPropagation();
        toggleMute();
        break;
      case 'f':
      case 'F':
        event.preventDefault();
        event.stopPropagation();
        toggleFullscreen();
        break;
      default:
        break;
    }
  };

  const handleTimeUpdate = () => {
    const now = Date.now();
    if (now - lastTimeUpdateRef.current < TIME_UPDATE_INTERVAL_MS) {
      return;
    }
    lastTimeUpdateRef.current = now;
    const video = videoRef.current;
    if (!video) {
      return;
    }
    setDuration(Number.isFinite(video.duration) ? video.duration : 0);
    setCurrentTime(video.currentTime);
  };

  const handleLoadedMetadata = () => {
    const video = videoRef.current;
    if (!video) {
      return;
    }
    setDuration(Number.isFinite(video.duration) ? video.duration : 0);
    setCurrentTime(video.currentTime);
  };

  const handleRetry = () => {
    setFailed(false);
    // 元素在失败状态下仍然挂载，所以 `load()` 一定作用在同一个 video 上：
    // 重新走一遍资源选择与解码，而不是靠重新挂载碰运气。
    videoRef.current?.load();
  };

  if (!url) {
    // 字节来源的首帧还没有 blob URL：先显示加载态，免得闪一下「加载失败」。
    return <PreviewLoadingPanel labels={labels} />;
  }

  const timeLabel = `${formatPlaybackTime(currentTime)} / ${formatPlaybackTime(duration)}`;

  return (
    <div
      ref={wrapperRef}
      aria-label={`${labels.preview}: ${name}`}
      className="flex h-full w-full flex-col bg-neutral-950"
      onKeyDown={handleKeyDown}
      role="group"
      tabIndex={0}
    >
      <div className={PREVIEW_TOOLBAR_CLASS}>
        <button
          aria-label={playing ? labels.pause : labels.play}
          aria-pressed={playing}
          className={PREVIEW_ICON_BUTTON_CLASS}
          onClick={togglePlayback}
          title={playing ? labels.pause : labels.play}
          type="button"
        >
          {playing ? <Pause aria-hidden="true" size={15} /> : <Play aria-hidden="true" size={15} />}
        </button>
        <span className="font-mono text-[11px] tabular-nums text-neutral-500 dark:text-neutral-400">
          {timeLabel}
        </span>
        <span className="min-w-0 flex-1 truncate text-xs text-neutral-500 dark:text-neutral-400" title={name}>
          {name}
        </span>
        <label className="flex items-center gap-1.5">
          <span className="text-[11px] text-neutral-500 dark:text-neutral-400">{labels.playbackRate}</span>
          <select
            aria-label={labels.playbackRate}
            className={PREVIEW_INPUT_CLASS}
            onChange={(event) => setPlaybackRate(Number(event.target.value))}
            title={labels.playbackRate}
            value={String(playbackRate)}
          >
            {PLAYBACK_RATES.map((rate) => (
              // 数字 + 乘号是跨语言通用的倍速写法，不进字典。
              <option key={rate} value={String(rate)}>{`${rate}×`}</option>
            ))}
          </select>
        </label>
        <button
          aria-label={labels.fullscreen}
          aria-pressed={fullscreen}
          className={PREVIEW_ICON_BUTTON_CLASS}
          onClick={toggleFullscreen}
          title={labels.fullscreen}
          type="button"
        >
          {fullscreen
            ? <Minimize2 aria-hidden="true" size={15} />
            : <Maximize2 aria-hidden="true" size={15} />}
        </button>
      </div>
      <div className={PREVIEW_CANVAS_CLASS}>
        <video
          ref={videoRef}
          aria-label={`${labels.preview}: ${name}`}
          className="max-h-full max-w-full"
          controls
          onEnded={() => setPlaying(false)}
          onError={() => setFailed(true)}
          onLoadedMetadata={handleLoadedMetadata}
          onPause={() => setPlaying(false)}
          onPlay={() => setPlaying(true)}
          onSeeked={handleTimeUpdate}
          onTimeUpdate={handleTimeUpdate}
          playsInline
          preload="metadata"
          src={url}
          title={name}
        />
        {failed ? (
          // 覆盖层而不是替换 video：元素一旦卸载，onRetry 里的 `load()` 就无处可调。
          <div className="absolute inset-0 bg-neutral-50 dark:bg-neutral-950">
            <PreviewErrorPanel labels={labels} message={labels.unavailableHint} onRetry={handleRetry} />
          </div>
        ) : null}
      </div>
    </div>
  );
}
