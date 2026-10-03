import React from 'react';
import { CircleAlert, HardDriveDownload, LoaderCircle, PackageOpen } from 'lucide-react';
import type { FilePreviewLabels } from '../i18n/filePreviewLabels';
import { formatFilePreviewLabel } from '../i18n/filePreviewLabels';
import { formatByteSize } from '../utils/bytes';
import {
  PREVIEW_PRIMARY_BUTTON_CLASS,
  PREVIEW_SECONDARY_BUTTON_CLASS,
} from './previewStyles';

export interface PreviewStatePanelProps {
  action?: React.ReactNode;
  description?: string;
  icon?: React.ReactNode;
  title: string;
}

/** 预览正文里的状态板：加载、空文件、失败、无法预览、超出上限共用同一骨架。 */
export function PreviewStatePanel({ action, description, icon, title }: PreviewStatePanelProps) {
  return (
    <div className="flex h-full w-full flex-col items-center justify-center gap-3 px-6 py-10 text-center">
      <div className="flex h-12 w-12 items-center justify-center rounded-full bg-neutral-100 text-neutral-500 dark:bg-neutral-800 dark:text-neutral-400">
        {icon}
      </div>
      <div className="max-w-md space-y-1">
        <p className="text-sm font-medium text-neutral-800 dark:text-neutral-100">{title}</p>
        {description ? (
          <p className="text-xs leading-relaxed text-neutral-500 dark:text-neutral-400">
            {description}
          </p>
        ) : null}
      </div>
      {action ? <div className="flex flex-wrap items-center justify-center gap-2">{action}</div> : null}
    </div>
  );
}

export function PreviewLoadingPanel({ labels }: { labels: FilePreviewLabels }) {
  return (
    <div aria-busy="true" role="status">
      <PreviewStatePanel
        icon={<LoaderCircle aria-hidden="true" className="animate-spin" size={20} />}
        title={labels.loading}
      />
    </div>
  );
}

export function PreviewEmptyPanel({ labels, name }: { labels: FilePreviewLabels; name: string }) {
  return (
    <PreviewStatePanel
      icon={<PackageOpen aria-hidden="true" size={20} />}
      title={labels.emptyFile}
      description={name}
    />
  );
}

export interface PreviewErrorPanelProps {
  labels: FilePreviewLabels;
  message: string;
  onRetry?: () => void;
}

export function PreviewErrorPanel({ labels, message, onRetry }: PreviewErrorPanelProps) {
  return (
    // `role="alert"`：读取失败必须被屏幕阅读器播报，否则用户只看到界面停住。
    <div role="alert">
      <PreviewStatePanel
        action={
          onRetry ? (
            <button type="button" className={PREVIEW_SECONDARY_BUTTON_CLASS} onClick={onRetry}>
              {labels.retry}
            </button>
          ) : null
        }
        description={message}
        icon={<CircleAlert aria-hidden="true" size={20} />}
        title={labels.loadFailed}
      />
    </div>
  );
}

export interface PreviewTooLargePanelProps {
  download?: () => void;
  labels: FilePreviewLabels;
  limitBytes: number;
  sizeBytes: number;
}

export function PreviewTooLargePanel({
  download,
  labels,
  limitBytes,
  sizeBytes,
}: PreviewTooLargePanelProps) {
  return (
    <div role="alert">
      <PreviewStatePanel
        action={
          download ? (
            <button type="button" className={PREVIEW_PRIMARY_BUTTON_CLASS} onClick={download}>
              <HardDriveDownload aria-hidden="true" size={14} />
              {labels.download}
            </button>
          ) : null
        }
        description={formatFilePreviewLabel(labels.tooLargeHint, {
          limit: formatByteSize(limitBytes),
          size: formatByteSize(sizeBytes),
        })}
        icon={<CircleAlert aria-hidden="true" size={20} />}
        title={labels.tooLargeTitle}
      />
    </div>
  );
}
