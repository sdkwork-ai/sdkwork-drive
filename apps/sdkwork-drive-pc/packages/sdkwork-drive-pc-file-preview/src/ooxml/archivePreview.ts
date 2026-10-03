/**
 * 通用归档（zip）列表预览。
 *
 * 与 OOXML 预览共用同一个 ZIP 读取层，但完全不碰条目内容：列目录只需要中央目录里的
 * 路径与尺寸，解压任何一个条目都是多余的开销，也会把压缩炸弹的体积风险带进来。
 */

import { readZipArchive } from './zipArchive';

/** 列表条目上限：再长的列表用户也只会滚动前几屏。 */
export const MAX_ARCHIVE_PREVIEW_ENTRIES = 2000;

export interface ArchivePreviewEntry {
  path: string;
  isDirectory: boolean;
  uncompressedSize: number;
  compressedSize: number;
}

export interface ArchivePreviewModel {
  entries: ArchivePreviewEntry[];
  totalUncompressedBytes: number;
  truncated: boolean;
}

/**
 * 读取归档目录。
 *
 * 总量按完整条目列表计算、截断只作用于返回的切片：调用方用总量显示「解压后多大」，
 * 若按截断后的列表求和会随上限变化，同一个文件在不同设置下显示不同体积。
 */
export function readArchivePreview(
  bytes: Uint8Array,
  options?: { maxEntries?: number },
): ArchivePreviewModel {
  const archive = readZipArchive(bytes);
  const entries: ArchivePreviewEntry[] = archive.entries.map((entry) => ({
    path: entry.path,
    isDirectory: entry.isDirectory,
    uncompressedSize: entry.uncompressedSize,
    compressedSize: entry.compressedSize,
  }));

  let totalUncompressedBytes = 0;
  for (const entry of entries) {
    totalUncompressedBytes += entry.uncompressedSize;
  }

  const sorted = [...entries].sort(compareArchiveEntries);
  const limit = resolveEntryLimit(options?.maxEntries);
  return {
    entries: sorted.slice(0, limit),
    totalUncompressedBytes,
    truncated: sorted.length > limit,
  };
}

/** 目录在前、文件在后，各自按路径升序。 */
function compareArchiveEntries(left: ArchivePreviewEntry, right: ArchivePreviewEntry): number {
  if (left.isDirectory !== right.isDirectory) {
    return left.isDirectory ? -1 : 1;
  }
  // 用码位比较而不是 `localeCompare`：归档列表的顺序不应随运行时 ICU 数据变化，
  // 否则同一份归档在不同语言环境下的展示顺序会不一致。
  if (left.path === right.path) {
    return 0;
  }
  return left.path < right.path ? -1 : 1;
}

function resolveEntryLimit(maxEntries: number | undefined): number {
  if (maxEntries === undefined || !Number.isFinite(maxEntries) || maxEntries < 0) {
    return MAX_ARCHIVE_PREVIEW_ENTRIES;
  }
  return Math.floor(maxEntries);
}
