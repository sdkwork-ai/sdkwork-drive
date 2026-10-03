import { useEffect, useRef, useState } from 'react';
import type { KeyboardEvent as ReactKeyboardEvent } from 'react';
import { Pause, Play, Volume2, VolumeX } from 'lucide-react';
import type { FilePreviewLabels } from '../../i18n/filePreviewLabels';
import { formatFilePreviewLabel } from '../../i18n/filePreviewLabels';
import { useMediaPreviewUrl, type MediaPreviewSource } from '../../hooks/useMediaPreviewUrl';
import { formatPlaybackTime } from '../../utils/formatPlaybackTime';
import { FileKindIcon } from '../FileKindIcon';
import { PreviewErrorPanel, PreviewLoadingPanel } from '../PreviewStatePanels';
import {
  PREVIEW_BADGE_CLASS,
  PREVIEW_ICON_BUTTON_CLASS,
  PREVIEW_INPUT_CLASS,
  PREVIEW_PANEL_CLASS,
} from '../previewStyles';

export interface AudioFilePreviewProps {
  labels: FilePreviewLabels;
  name: string;
  source: MediaPreviewSource;
}

/** 播放速度档位，与视频预览保持一致。 */
const PLAYBACK_RATES = [0.5, 1, 1.25, 1.5, 2] as const;

/** 与视频预览同一档节流：读数够跟手，又不让工具栏随 timeupdate 高频重渲。 */
const TIME_UPDATE_INTERVAL_MS = 250;

const INTERACTIVE_TARGET_SELECTOR =
  'button, input, select, textarea, a[href], [role="button"], [role="slider"], [role="tab"]';

function startPlayback(element: HTMLMediaElement): void {
  try {
    const pending: unknown = element.play();
    if (pending instanceof Promise) {
      pending.catch(() => undefined);
    }
  } catch {
    // 宿主不支持播放（含 jsdom）：忽略，界面状态由 onError 反映。
  }
}

function toPercent(value: number, total: number): number {
  if (!Number.isFinite(value) || !Number.isFinite(total) || total <= 0) {
    return 0;
  }
  return Math.min(Math.max((value / total) * 100, 0), 100);
}

/**
 * 音频预览：原生 `<audio>` 只当解码器，界面用自绘控制条。
 *
 * 原生 `<audio controls>` 在 Chromium/Firefox/Safari 里是三种不同外观，且都不提供
 * 缓冲进度可视化；音频预览的视觉重心是文件名与封面图块，控件必须和它们排在同一套
 * 布局里，所以这里要自己画控制条，但播放/缓冲/音量仍交给原生元素。
 */
export function AudioFilePreview({ labels, name, source }: AudioFilePreviewProps) {
  const url = useMediaPreviewUrl(source);
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const lastTimeUpdateRef = useRef(0);
  const [failed, setFailed] = useState(false);
  const [playing, setPlaying] = useState(false);
  const [muted, setMuted] = useState(false);
  const [volume, setVolume] = useState(1);
  const [playbackRate, setPlaybackRate] = useState(1);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [bufferedTime, setBufferedTime] = useState(0);

  // 依赖里带上 url：audio 元素在 URL 就绪后才挂载。
  useEffect(() => {
    const audio = audioRef.current;
    if (audio) {
      audio.volume = volume;
      audio.muted = muted;
    }
  }, [muted, url, volume]);

  useEffect(() => {
    const audio = audioRef.current;
    if (audio) {
      audio.playbackRate = playbackRate;
    }
  }, [playbackRate, url]);

  const togglePlayback = () => {
    const audio = audioRef.current;
    if (!audio) {
      return;
    }
    if (playing) {
      audio.pause();
      setPlaying(false);
      return;
    }
    // 状态先行，再由 onPlay/onPause 校正：`paused` 在部分宿主里不随 play() 同步。
    setPlaying(true);
    startPlayback(audio);
  };

  const toggleMute = () => setMuted((value) => !value);

  const handleVolumeChange = (nextVolume: number) => {
    setVolume(nextVolume);
    if (nextVolume > 0 && muted) {
      // 用户主动调大音量即视为取消静音，否则会出现「拖了滑块还是没声音」。
      setMuted(false);
    }
  };

  const handleSeek = (nextTime: number) => {
    const audio = audioRef.current;
    if (!audio) {
      return;
    }
    audio.currentTime = nextTime;
    setCurrentTime(nextTime);
  };

  const readBufferedTime = () => {
    const audio = audioRef.current;
    if (!audio) {
      return 0;
    }
    let furthest = 0;
    for (let index = 0; index < audio.buffered.length; index += 1) {
      furthest = Math.max(furthest, audio.buffered.end(index));
    }
    return furthest;
  };

  const handleTimeUpdate = () => {
    const now = Date.now();
    if (now - lastTimeUpdateRef.current < TIME_UPDATE_INTERVAL_MS) {
      return;
    }
    lastTimeUpdateRef.current = now;
    const audio = audioRef.current;
    if (!audio) {
      return;
    }
    setDuration(Number.isFinite(audio.duration) ? audio.duration : 0);
    setCurrentTime(audio.currentTime);
    setBufferedTime(readBufferedTime());
  };

  const handleLoadedMetadata = () => {
    const audio = audioRef.current;
    if (!audio) {
      return;
    }
    setDuration(Number.isFinite(audio.duration) ? audio.duration : 0);
    setCurrentTime(audio.currentTime);
    setBufferedTime(readBufferedTime());
  };

  const handleKeyDown = (event: ReactKeyboardEvent<HTMLDivElement>) => {
    if (event.target instanceof Element && event.target.closest(INTERACTIVE_TARGET_SELECTOR) !== null) {
      return;
    }
    if (event.key === ' ' || event.key === 'Spacebar') {
      event.preventDefault();
      // 播放器消费掉的按键不再冒泡：预览层用方向键切换文件，空格键必须留在播放器里。
      event.stopPropagation();
      togglePlayback();
    }
  };

  if (!url) {
    // 字节来源的首帧还没有 blob URL，先给加载态而不是一帧「加载失败」。
    return <PreviewLoadingPanel labels={labels} />;
  }

  const timeLabel = `${formatPlaybackTime(currentTime)} / ${formatPlaybackTime(duration)}`;
  const playedPercent = toPercent(currentTime, duration);
  const bufferedPercent = Math.max(toPercent(bufferedTime, duration), playedPercent);

  return (
    <div
      aria-label={`${labels.preview}: ${name}`}
      className="flex h-full w-full flex-col items-center justify-center gap-6 overflow-auto bg-neutral-50 px-6 py-10 dark:bg-neutral-950"
      onKeyDown={handleKeyDown}
      role="group"
      tabIndex={0}
    >
      <audio
        ref={audioRef}
        aria-label={`${labels.preview}: ${name}`}
        className="sr-only"
        onEnded={() => setPlaying(false)}
        onError={() => setFailed(true)}
        onLoadedMetadata={handleLoadedMetadata}
        onPause={() => setPlaying(false)}
        onPlay={() => setPlaying(true)}
        onProgress={handleTimeUpdate}
        onSeeked={handleTimeUpdate}
        onTimeUpdate={handleTimeUpdate}
        preload="metadata"
        src={url}
      />

      <div className="flex min-w-0 flex-col items-center gap-3 text-center">
        <FileKindIcon kind="audio" size="xl" />
        <p
          className="max-w-md truncate text-sm font-medium text-neutral-800 dark:text-neutral-100"
          title={name}
        >
          {name}
        </p>
        <span className={PREVIEW_BADGE_CLASS}>{labels.kind.audio}</span>
      </div>

      {failed ? (
        <div className={`w-full max-w-xl ${PREVIEW_PANEL_CLASS}`}>
          <PreviewErrorPanel
            labels={labels}
            message={labels.unavailableHint}
            onRetry={() => {
              setFailed(false);
              audioRef.current?.load();
            }}
          />
        </div>
      ) : (
        <div className={`w-full max-w-xl p-4 ${PREVIEW_PANEL_CLASS}`}>
          <div className="flex items-center gap-3">
            <button
              aria-label={playing ? labels.pause : labels.play}
              aria-pressed={playing}
              className={PREVIEW_ICON_BUTTON_CLASS}
              onClick={togglePlayback}
              title={playing ? labels.pause : labels.play}
              type="button"
            >
              {playing ? <Pause aria-hidden="true" size={16} /> : <Play aria-hidden="true" size={16} />}
            </button>

            <span className="shrink-0 font-mono text-[11px] tabular-nums text-neutral-500 dark:text-neutral-400">
              {timeLabel}
            </span>

            {/* 缓冲与已播放进度用两层绝对定位的色条叠出来：range 的原生轨道只有一种颜色，
                Tailwind 又无法表达「按百分比分割」的背景，所以宽度走行内 style，颜色仍走类名。 */}
            <div className="relative h-4 min-w-0 flex-1">
              <div className="absolute inset-x-0 top-1/2 h-1.5 -translate-y-1/2 overflow-hidden rounded-full bg-neutral-200 dark:bg-neutral-700">
                <div
                  className="h-full rounded-full bg-neutral-300 transition-[width] duration-200 dark:bg-neutral-600"
                  style={{ width: `${bufferedPercent}%` }}
                />
              </div>
              <div
                className="pointer-events-none absolute left-0 top-1/2 h-1.5 -translate-y-1/2 rounded-full bg-blue-500 dark:bg-blue-400"
                style={{ width: `${playedPercent}%` }}
              />
              <input
                // 进度条的可访问名复用「当前 / 总数」模板：标签表里没有专门的「播放进度」
                // 词条，硬编码一个英文词会破坏「文案全部来自 labels」的约束。
                aria-label={formatFilePreviewLabel(labels.matchCount, {
                  current: formatPlaybackTime(currentTime),
                  total: formatPlaybackTime(duration),
                })}
                className="absolute inset-0 h-full w-full cursor-pointer appearance-none bg-transparent [&::-moz-range-thumb]:h-3 [&::-moz-range-thumb]:w-3 [&::-moz-range-thumb]:rounded-full [&::-moz-range-thumb]:border-0 [&::-moz-range-thumb]:bg-blue-600 dark:[&::-moz-range-thumb]:bg-blue-400 [&::-moz-range-track]:bg-transparent [&::-webkit-slider-runnable-track]:bg-transparent [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-blue-600 dark:[&::-webkit-slider-thumb]:bg-blue-400"
                max={duration > 0 ? duration : 0}
                min={0}
                onChange={(event) => handleSeek(Number(event.target.value))}
                step={0.1}
                type="range"
                value={duration > 0 ? Math.min(currentTime, duration) : 0}
              />
            </div>

            <button
              aria-label={muted ? labels.unmute : labels.mute}
              aria-pressed={muted}
              className={PREVIEW_ICON_BUTTON_CLASS}
              onClick={toggleMute}
              title={muted ? labels.unmute : labels.mute}
              type="button"
            >
              {muted ? <VolumeX aria-hidden="true" size={15} /> : <Volume2 aria-hidden="true" size={15} />}
            </button>
            <input
              aria-label={labels.volume}
              className="h-1.5 w-20 shrink-0 cursor-pointer appearance-none rounded-full bg-neutral-200 accent-blue-600 dark:bg-neutral-700 dark:accent-blue-400"
              max={1}
              min={0}
              onChange={(event) => handleVolumeChange(Number(event.target.value))}
              step={0.05}
              title={labels.volume}
              type="range"
              value={muted ? 0 : volume}
            />
          </div>

          <div className="mt-3 flex items-center gap-2">
            <span className="text-[11px] text-neutral-500 dark:text-neutral-400">{labels.playbackRate}</span>
            <select
              aria-label={labels.playbackRate}
              className={PREVIEW_INPUT_CLASS}
              onChange={(event) => setPlaybackRate(Number(event.target.value))}
              title={labels.playbackRate}
              value={String(playbackRate)}
            >
              {PLAYBACK_RATES.map((rate) => (
                // 数字 + 乘号语言中立，不进字典。
                <option key={rate} value={String(rate)}>{`${rate}×`}</option>
              ))}
            </select>
          </div>
        </div>
      )}
    </div>
  );
}
