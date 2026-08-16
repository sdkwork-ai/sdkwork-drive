import {
  encodeLocalFilesystemId,
  type LocalFilesystemEntry,
  type DriveFile,
} from '../types';

export function mapLocalFilesystemEntryToDriveFile(
  entry: LocalFilesystemEntry,
  ownerId: string,
  parentId?: string | null,
): DriveFile {
  return {
    id: encodeLocalFilesystemId(entry.path),
    name: entry.name,
    type: entry.isDirectory ? 'folder' : 'file',
    mimeType: entry.mimeType,
    size: entry.size,
    updatedAt: entry.modifiedAt ?? new Date(0).toISOString(),
    ownerId,
    parentId: parentId ?? undefined,
  };
}

function splitLocalFilesystemPath(folderPath: string): { segments: string[]; useBackslashes: boolean } {
  const trimmed = folderPath.trim();
  if (!trimmed) {
    return { segments: [], useBackslashes: false };
  }

  const useBackslashes = trimmed.includes('\\') || /^[A-Za-z]:[\\/]?/.test(trimmed);

  if (/^[A-Za-z]:[\\/]?$/.test(trimmed)) {
    return { segments: [trimmed.slice(0, 2).toUpperCase()], useBackslashes: true };
  }

  if (/^[A-Za-z]:[\\/]/.test(trimmed)) {
    const drive = trimmed.slice(0, 2).toUpperCase();
    const rest = trimmed.slice(2).replace(/^[\\/]+/, '');
    const parts = rest ? rest.split(/[\\/]+/).filter(Boolean) : [];
    return { segments: [drive, ...parts], useBackslashes: true };
  }

  if (trimmed.startsWith('\\\\')) {
    const parts = trimmed.slice(2).split(/[\\/]+/).filter(Boolean);
    return { segments: parts, useBackslashes: true };
  }

  const normalized = trimmed.replace(/\\/g, '/');
  const parts = normalized.split('/').filter(Boolean);
  return { segments: parts, useBackslashes: false };
}

function joinLocalFilesystemSegment(
  currentPath: string,
  segment: string,
  useBackslashes: boolean,
  isFirstSegment: boolean,
): string {
  if (isFirstSegment) {
    if (/^[A-Za-z]:$/.test(segment)) {
      return `${segment}\\`;
    }
    return useBackslashes ? segment : `/${segment}`;
  }

  if (/^[A-Za-z]:\\?$/.test(currentPath)) {
    return `${currentPath.replace(/\\?$/, '\\')}${segment}`;
  }

  const separator = useBackslashes ? '\\' : '/';
  if (currentPath.endsWith(separator)) {
    return `${currentPath}${segment}`;
  }
  return `${currentPath}${separator}${segment}`;
}

export function buildLocalFilesystemFolderPath(
  folderPath: string,
  ownerId: string,
): DriveFile[] {
  const { segments, useBackslashes } = splitLocalFilesystemPath(folderPath);
  if (segments.length === 0) {
    return [];
  }

  const crumbs: DriveFile[] = [];
  let currentPath = '';

  segments.forEach((segment, index) => {
    currentPath = joinLocalFilesystemSegment(currentPath, segment, useBackslashes, index === 0);
    crumbs.push({
      id: encodeLocalFilesystemId(currentPath),
      name: segment,
      type: 'folder',
      updatedAt: new Date(0).toISOString(),
      ownerId,
      parentId: crumbs.at(-1)?.id,
    });
  });

  return crumbs;
}
