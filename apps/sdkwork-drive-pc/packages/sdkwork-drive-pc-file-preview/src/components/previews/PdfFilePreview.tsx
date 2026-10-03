import React, { useState } from 'react';
import { ExternalLink } from 'lucide-react';
import type { FilePreviewLabels } from '../../i18n/filePreviewLabels';
import { useMediaPreviewUrl, type MediaPreviewSource } from '../../hooks/useMediaPreviewUrl';
import { PreviewErrorPanel, PreviewLoadingPanel } from '../PreviewStatePanels';
import { PREVIEW_GHOST_BUTTON_CLASS } from '../previewStyles';

export interface PdfFilePreviewProps {
  labels: FilePreviewLabels;
  name: string;
  source: MediaPreviewSource;
}

/**
 * PDF 预览：交给浏览器内置的 PDF 阅读器（`<iframe>` + blob/直链）。
 *
 * 自绘 PDF 渲染器要引入 pdf.js 与它自己的 worker/字体资源，而浏览器自带的阅读器
 * 已经提供滚动、缩放、检索与打印。这里只负责把资源变成可寻址 URL，并在内置阅读器
 * 不可用时给出「新窗口打开」的退路。
 */
export function PdfFilePreview({ labels, name, source }: PdfFilePreviewProps) {
  const url = useMediaPreviewUrl(source);
  const [failed, setFailed] = useState(false);

  if (!url) {
    // 直链/字节转 blob URL 还没完成：这是加载态，不是错误（视频与音频预览同样处理）。
    return <PreviewLoadingPanel labels={labels} />;
  }

  return (
    <div className="flex h-full w-full flex-col bg-neutral-100 dark:bg-neutral-900">
      <div className="flex shrink-0 items-center gap-2 border-b border-neutral-200 bg-white/90 px-3 py-1.5 dark:border-neutral-800 dark:bg-neutral-900/90">
        <span className="min-w-0 flex-1 truncate font-mono text-[11px] text-neutral-500 dark:text-neutral-400">
          {name}
        </span>
        <a
          className={PREVIEW_GHOST_BUTTON_CLASS}
          href={url}
          rel="noreferrer"
          target="_blank"
        >
          <ExternalLink aria-hidden="true" size={13} />
          {labels.openInNewTab}
        </a>
      </div>
      {failed ? (
        <PreviewErrorPanel
          labels={labels}
          message={labels.unavailableHint}
          onRetry={() => setFailed(false)}
        />
      ) : (
        <object
          aria-label={`${labels.preview}: ${name}`}
          className="min-h-0 flex-1"
          data={url}
          type="application/pdf"
          onError={() => setFailed(true)}
        >
          <iframe className="h-full w-full" src={url} title={name} />
        </object>
      )}
    </div>
  );
}
