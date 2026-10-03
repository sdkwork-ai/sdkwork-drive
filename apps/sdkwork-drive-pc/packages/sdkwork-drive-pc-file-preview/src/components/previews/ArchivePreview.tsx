import { useMemo, useState } from 'react';
import type { ArchivePreviewModel } from '../../ooxml/archivePreview';
import { readArchivePreview } from '../../ooxml/archivePreview';
import type { FilePreviewLabels } from '../../i18n/filePreviewLabels';
import { formatFilePreviewLabel } from '../../i18n/filePreviewLabels';
import { FileKindIcon } from '../FileKindIcon';
import { resolveFilePreviewKind } from '../../kinds/filePreviewKind';
import { formatByteSize } from '../../utils/bytes';
import { describePreviewError } from '../../utils/describePreviewError';
import { PreviewEmptyPanel, PreviewErrorPanel } from '../PreviewStatePanels';
import {
  PREVIEW_BADGE_CLASS,
  PREVIEW_SCROLL_CLASS,
  PREVIEW_TOOLBAR_CLASS,
} from '../previewStyles';

export interface ArchivePreviewProps {
  bytes: Uint8Array;
  labels: FilePreviewLabels;
  name: string;
}

type ArchiveEntryModel = ArchivePreviewModel['entries'][number];

type ArchiveReadResult =
  | { ok: true; model: ArchivePreviewModel }
  | { ok: false; message: string };

/**
 * 目录在前、其余按路径排序。
 *
 * 当前的 ZIP 解析器已经按同样规则排过序，这里仍然再排一次：渲染顺序属于视图的职责，
 * 一旦解析后端换成 7z/tar 或条目来自别处，顺序不该跟着变；列表上限 2000 条，排序
 * 又被 `useMemo` 记住，代价可以忽略。
 */
function compareEntries(left: ArchiveEntryModel, right: ArchiveEntryModel): number {
  if (left.isDirectory !== right.isDirectory) {
    return left.isDirectory ? -1 : 1;
  }
  return left.path.localeCompare(right.path);
}

/**
 * 压缩包预览：列出条目清单。
 *
 * 只列清单、不提供解压到本地：预览层没有写文件的能力，而条目名、目录结构与压缩率
 * 已经足够回答「这个包里是什么、值不值得下载」。解析是同步的（纯内存 ZIP 目录读取，
 * 通常几毫秒），但仍在 `useMemo` 里包一层 try/catch，让坏包变成一个错误面板而不是
 * 一次渲染崩溃。
 */
export function ArchivePreview({ bytes, labels, name }: ArchivePreviewProps) {
  const [attempt, setAttempt] = useState(0);

  const result = useMemo<ArchiveReadResult>(() => {
    try {
      return { ok: true, model: readArchivePreview(bytes) };
    } catch (cause: unknown) {
      return { ok: false, message: describePreviewError(cause, labels) };
    }
    // 只依赖字节数组本身：宿主从内容层拿到的是一份稳定引用（见 useFilePreviewContent），
    // 按引用记忆化即可，不需要再按内容特征做身份。
  }, [attempt, bytes, labels]);

  const sortedEntries = useMemo<ArchiveEntryModel[]>(() => {
    if (!result.ok) {
      return [];
    }
    return [...result.model.entries].sort(compareEntries);
  }, [result]);

  if (!result.ok) {
    return (
      <PreviewErrorPanel
        labels={labels}
        message={result.message}
        onRetry={() => setAttempt((value) => value + 1)}
      />
    );
  }

  const { model } = result;
  const entryCount = model.entries.length;
  if (entryCount === 0) {
    return <PreviewEmptyPanel labels={labels} name={name} />;
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className={PREVIEW_TOOLBAR_CLASS}>
        <span className="min-w-0 flex-1 truncate text-xs text-neutral-500 dark:text-neutral-400" title={name}>
          {name}
        </span>
        <span className={PREVIEW_BADGE_CLASS}>
          {formatFilePreviewLabel(labels.archiveEntries, { count: entryCount })}
        </span>
        <span className={PREVIEW_BADGE_CLASS}>
          {formatFilePreviewLabel(labels.archiveTotalSize, {
            size: formatByteSize(model.totalUncompressedBytes),
          })}
        </span>
      </div>

      <div className={`${PREVIEW_SCROLL_CLASS} bg-white dark:bg-neutral-950`}>
        <table className="w-full border-collapse text-left text-xs">
          <tbody>
            {sortedEntries.map((entry, index) => (
              <tr
                key={`${entry.path}-${index}`}
                className="border-b border-neutral-100 last:border-b-0 dark:border-neutral-800"
              >
                {/* 路径即行标题：表格里第一列是条目标识而不是数据列，用 th[scope=row] 让
                    屏幕阅读器把后面的尺寸读成「路径 → 尺寸」而不是一串数字。 */}
                <th className="px-3 py-1.5 text-left font-normal" scope="row">
                  <span className="flex min-w-0 items-center gap-2">
                    <FileKindIcon
                      kind={resolveFilePreviewKind({ name: entry.path, isFolder: entry.isDirectory })}
                      size="sm"
                    />
                    <span
                      className={
                        entry.isDirectory
                          ? 'min-w-0 truncate font-mono text-[12px] font-medium text-neutral-900 dark:text-neutral-100'
                          : 'min-w-0 truncate font-mono text-[12px] text-neutral-600 dark:text-neutral-300'
                      }
                      title={entry.path}
                    >
                      {entry.path}
                    </span>
                  </span>
                </th>
                <td className="whitespace-nowrap px-3 py-1.5 text-right tabular-nums text-neutral-500 dark:text-neutral-400">
                  {/* 目录自身不占空间：显示长度会让「包多大」看起来比实际大一圈。
                      未压缩尺寸是主信息，压缩后尺寸作为次信息跟在后面。 */}
                  {formatByteSize(entry.isDirectory ? undefined : entry.uncompressedSize)}
                  {entry.isDirectory ? null : (
                    <span className="ml-2 text-[11px] text-neutral-400 dark:text-neutral-500">
                      {formatByteSize(entry.compressedSize)}
                    </span>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {model.truncated ? (
        <p className="shrink-0 border-t border-amber-200 bg-amber-50 px-3 py-1.5 text-center text-[11px] text-amber-700 dark:border-amber-900/50 dark:bg-amber-950/30 dark:text-amber-300">
          {formatFilePreviewLabel(labels.archiveTruncated, { count: entryCount })}
        </p>
      ) : null}
    </div>
  );
}
