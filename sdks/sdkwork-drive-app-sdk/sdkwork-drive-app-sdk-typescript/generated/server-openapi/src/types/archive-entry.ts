export interface ArchiveEntry {
  path: string;
  name: string;
  isDirectory: boolean;
  uncompressedSizeBytes: string;
  compressedSizeBytes: string;
  contentType?: string;
}
