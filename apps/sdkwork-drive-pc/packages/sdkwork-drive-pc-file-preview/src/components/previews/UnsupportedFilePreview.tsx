import React from 'react';
import { Download, FileQuestion } from 'lucide-react';
import type { FilePreviewLabels } from '../../i18n/filePreviewLabels';
import { formatByteSize } from '../../utils/bytes';
import { PREVIEW_PRIMARY_BUTTON_CLASS } from '../previewStyles';

export interface UnsupportedFilePreviewProps {
  contentType?: string;
  download?: () => void;
  labels: FilePreviewLabels;
  name: string;
  sizeBytes?: number;
}

/**
 * 兜底预览：说清楚「这个类型暂时没有内联预览」，并给出文件信息与下载入口。
 *
 * 一个诚实的兜底比一个假装能打开的空面板有价值：用户至少知道文件多大、什么类型、
 * 下一步该做什么。
 */
export function UnsupportedFilePreview({
  contentType,
  download,
  labels,
  name,
  sizeBytes,
}: UnsupportedFilePreviewProps) {
  return (
    <div className="flex h-full w-full flex-col items-center justify-center gap-4 px-6 py-10 text-center">
      <div className="flex h-12 w-12 items-center justify-center rounded-full bg-neutral-100 text-neutral-500 dark:bg-neutral-800 dark:text-neutral-400">
        <FileQuestion aria-hidden="true" size={20} />
      </div>
      <div className="space-y-1">
        <p className="max-w-md break-all text-sm font-medium text-neutral-800 dark:text-neutral-100">
          {name}
        </p>
        <p className="max-w-md text-xs leading-relaxed text-neutral-500 dark:text-neutral-400">
          {labels.unavailableHint}
        </p>
      </div>
      <FileMetadataList
        contentType={contentType}
        labels={labels}
        sizeBytes={sizeBytes}
      />
      {download ? (
        <button type="button" className={PREVIEW_PRIMARY_BUTTON_CLASS} onClick={download}>
          <Download aria-hidden="true" size={14} />
          {labels.download}
        </button>
      ) : null}
    </div>
  );
}

export interface FileMetadataListProps {
  contentType?: string;
  labels: FilePreviewLabels;
  sizeBytes?: number;
}

/** 兜底面板的元信息表：类型与大小。 */
export function FileMetadataList({ contentType, labels, sizeBytes }: FileMetadataListProps) {
  const rows: Array<[string, string]> = [
    [labels.typeLabel, contentType ?? '—'],
    [labels.sizeLabel, formatByteSize(sizeBytes)],
  ];
  return (
    <dl className="grid w-full max-w-md gap-1.5 text-left text-xs">
      {rows.map(([label, value]) => (
        <div key={label} className="flex items-start gap-2">
          <dt className="w-16 shrink-0 text-neutral-400">{label}</dt>
          <dd className="min-w-0 flex-1 break-all text-neutral-700 dark:text-neutral-200">{value}</dd>
        </div>
      ))}
    </dl>
  );
}
