import type { DriveFile } from 'sdkwork-drive-pc-types';

export function isDefaultFileBrowserSort(
  sortBy: string,
  sortOrder: string,
): boolean {
  return sortBy === 'name' && sortOrder === 'asc';
}

export function mergeUniqueDriveFiles(
  current: readonly DriveFile[],
  incoming: readonly DriveFile[],
): DriveFile[] {
  const seen = new Set(current.map((file) => file.id));
  const merged = [...current];
  for (const file of incoming) {
    if (!seen.has(file.id)) {
      merged.push(file);
      seen.add(file.id);
    }
  }
  return merged;
}
