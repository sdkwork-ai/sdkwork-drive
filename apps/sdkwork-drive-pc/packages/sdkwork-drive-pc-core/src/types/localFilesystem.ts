export const LOCAL_FILESYSTEM_ID_PREFIX = 'local-fs:';

export type LocalFilesystemEntryKind =
  | 'drive'
  | 'root'
  | 'home'
  | 'desktop'
  | 'documents'
  | 'downloads'
  | 'folder'
  | 'file';

export interface LocalFilesystemEntry {
  name: string;
  path: string;
  isDirectory: boolean;
  size?: number;
  modifiedAt?: string;
  mimeType?: string;
  entryKind: LocalFilesystemEntryKind;
}

export function encodeLocalFilesystemId(path: string): string {
  return `${LOCAL_FILESYSTEM_ID_PREFIX}${encodeURIComponent(path)}`;
}

export function decodeLocalFilesystemId(id: string): string | null {
  if (!id.startsWith(LOCAL_FILESYSTEM_ID_PREFIX)) {
    return null;
  }
  try {
    return decodeURIComponent(id.slice(LOCAL_FILESYSTEM_ID_PREFIX.length));
  } catch {
    return null;
  }
}

export function isLocalFilesystemDriveFileId(id: string): boolean {
  return id.startsWith(LOCAL_FILESYSTEM_ID_PREFIX);
}
