import { useEffect, useState } from 'react';
import type { KeyboardEvent as ReactKeyboardEvent } from 'react';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import type { PresentationPreviewModel } from '../../ooxml/presentationPreview';
import { parsePresentation } from '../../ooxml/presentationPreview';
import type { FilePreviewLabels } from '../../i18n/filePreviewLabels';
import { formatFilePreviewLabel } from '../../i18n/filePreviewLabels';
import { describePreviewError } from '../../utils/describePreviewError';
import { PreviewEmptyPanel, PreviewErrorPanel, PreviewLoadingPanel } from '../PreviewStatePanels';
import {
  PREVIEW_BADGE_CLASS,
  PREVIEW_GHOST_BUTTON_CLASS,
  PREVIEW_SCROLL_CLASS,
  PREVIEW_TOOLBAR_CLASS,
} from '../previewStyles';

export interface SlidesPreviewProps {
  bytes: Uint8Array;
  labels: FilePreviewLabels;
  name: string;
}

type SlideModel = PresentationPreviewModel['slides'][number];

const INTERACTIVE_TARGET_SELECTOR =
  'button, input, select, textarea, a[href], [role="button"], [role="slider"], [role="tab"]';

const ACTIVE_RAIL_ITEM_CLASS =
  'flex w-full items-center gap-2 rounded-md bg-blue-50 px-2 py-1.5 text-left text-xs font-medium text-blue-700 dark:bg-blue-950/40 dark:text-blue-300';

const INACTIVE_RAIL_ITEM_CLASS =
  'flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-xs text-neutral-600 transition-colors hover:bg-neutral-100 hover:text-neutral-900 dark:text-neutral-300 dark:hover:bg-neutral-800 dark:hover:text-white';

/** 无标题的幻灯片用「幻灯片 N」兜底，轨道里不会出现一行空白按钮。 */
function slideLabel(slide: SlideModel, labels: FilePreviewLabels): string {
  const title = slide.title?.trim();
  return title && title !== '' ? title : `${labels.slideLabel} ${slide.index}`;
}

/**
 * 幻灯片预览：左侧缩略列表 + 右侧单页画布。
 *
 * 只渲染文字（标题 + 要点），不还原主题、图片与动画：`.pptx` 的视觉信息几乎都在
 * 主题文件与 DrawingML 里，够不上「一次预览」的成本。文字大纲已经能回答用户最常问的
 * 问题——「这份演示讲什么、一共几页」。
 */
export function SlidesPreview({ bytes, labels, name }: SlidesPreviewProps) {
  const [model, setModel] = useState<PresentationPreviewModel | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [pending, setPending] = useState(true);
  const [attempt, setAttempt] = useState(0);
  // 只存用户的翻页选择；默认停在第 1 页属于派生值。
  const [selected, setSelected] = useState<number | undefined>(undefined);

  useEffect(() => {
    let active = true;
    setPending(true);
    setError(undefined);

    parsePresentation(bytes)
      .then((result) => {
        if (!active) {
          return;
        }
        setModel(result);
        setSelected(undefined);
        setPending(false);
      })
      .catch((cause: unknown) => {
        if (!active) {
          return;
        }
        setModel(undefined);
        setError(describePreviewError(cause, labels));
        setPending(false);
      });

    return () => {
      active = false;
    };
  }, [attempt, bytes, labels.loadFailed]);

  if (error !== undefined) {
    return (
      <PreviewErrorPanel
        labels={labels}
        message={error}
        onRetry={() => setAttempt((value) => value + 1)}
      />
    );
  }

  if (pending || !model) {
    return <PreviewLoadingPanel labels={labels} />;
  }

  const slideCount = model.slides.length;
  if (slideCount === 0) {
    return <PreviewEmptyPanel labels={labels} name={name} />;
  }

  const activeIndex = Math.min(Math.max(selected ?? 0, 0), slideCount - 1);
  const activeSlide = model.slides[activeIndex];

  const goTo = (index: number) => setSelected(Math.min(Math.max(index, 0), slideCount - 1));

  const handleKeyDown = (event: ReactKeyboardEvent<HTMLDivElement>) => {
    if (event.target instanceof Element && event.target.closest(INTERACTIVE_TARGET_SELECTOR) !== null) {
      // 焦点在按钮/导航控件上时方向键属于它们，抢键会让用户按两次才翻一页。
      return;
    }
    if (event.key === 'ArrowLeft') {
      event.preventDefault();
      goTo(activeIndex - 1);
      return;
    }
    if (event.key === 'ArrowRight') {
      event.preventDefault();
      goTo(activeIndex + 1);
    }
  };

  return (
    <div
      aria-label={`${labels.preview}: ${name}`}
      className="flex min-h-0 flex-1 flex-col"
      onKeyDown={handleKeyDown}
      role="group"
      // 键盘翻页只在面板自己持有焦点时生效：预览常嵌在抽屉里，全局监听会劫持宿主快捷键。
      tabIndex={0}
    >
      <div className={PREVIEW_TOOLBAR_CLASS}>
        <span className="min-w-0 flex-1 truncate text-xs text-neutral-500 dark:text-neutral-400" title={name}>
          {name}
        </span>
        <span className={PREVIEW_BADGE_CLASS}>
          {formatFilePreviewLabel(labels.slideCount, { count: slideCount })}
        </span>
        <button
          aria-label={labels.previous}
          className={PREVIEW_GHOST_BUTTON_CLASS}
          disabled={activeIndex === 0}
          onClick={() => goTo(activeIndex - 1)}
          title={labels.previous}
          type="button"
        >
          <ChevronLeft aria-hidden="true" size={14} />
        </button>
        <button
          aria-label={labels.next}
          className={PREVIEW_GHOST_BUTTON_CLASS}
          disabled={activeIndex === slideCount - 1}
          onClick={() => goTo(activeIndex + 1)}
          title={labels.next}
          type="button"
        >
          <ChevronRight aria-hidden="true" size={14} />
        </button>
      </div>

      <div className="flex min-h-0 flex-1">
        <div className="w-40 shrink-0 space-y-1 overflow-y-auto border-r border-neutral-200 bg-white p-2 dark:border-neutral-800 dark:bg-neutral-900">
          {model.slides.map((slide, index) => {
            const label = slideLabel(slide, labels);
            return (
              <button
                key={`${slide.index}-${index}`}
                aria-current={index === activeIndex ? 'true' : undefined}
                aria-label={`${labels.slideLabel} ${slide.index}: ${label}`}
                className={index === activeIndex ? ACTIVE_RAIL_ITEM_CLASS : INACTIVE_RAIL_ITEM_CLASS}
                onClick={() => goTo(index)}
                title={label}
                type="button"
              >
                {/* 编号只是视觉定位，可访问名已经用 aria-label 完整表述。 */}
                <span
                  aria-hidden="true"
                  className="w-5 shrink-0 text-right font-mono text-[11px] tabular-nums text-neutral-400 dark:text-neutral-500"
                >
                  {slide.index}
                </span>
                <span className="min-w-0 truncate">{label}</span>
              </button>
            );
          })}
        </div>

        <div className={`${PREVIEW_SCROLL_CLASS} bg-neutral-100 p-4 dark:bg-neutral-950 sm:p-6`}>
          {activeSlide ? (
            <div className="mx-auto aspect-video w-full max-w-3xl overflow-auto rounded-lg border border-neutral-200 bg-white p-6 shadow-sm dark:border-neutral-800 dark:bg-neutral-900 sm:p-8">
              {activeSlide.title ? (
                <h2 className="mb-4 text-lg font-semibold text-neutral-900 dark:text-neutral-50">
                  {activeSlide.title}
                </h2>
              ) : null}
              <ul className="space-y-2 text-sm leading-6 text-neutral-700 dark:text-neutral-200">
                {activeSlide.lines.map((line, lineIndex) => (
                  <li key={lineIndex} className="flex gap-2">
                    <span aria-hidden="true" className="select-none text-neutral-400 dark:text-neutral-500">•</span>
                    <span className="min-w-0">{line}</span>
                  </li>
                ))}
              </ul>
            </div>
          ) : null}
        </div>
      </div>
    </div>
  );
}
