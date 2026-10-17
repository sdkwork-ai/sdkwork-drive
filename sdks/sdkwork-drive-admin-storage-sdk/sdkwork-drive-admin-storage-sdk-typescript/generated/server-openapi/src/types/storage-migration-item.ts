export interface StorageMigrationItem {
  id: string;
  storageObjectId: string;
  sourceProviderId: string;
  sourceBucket: string;
  sourceObjectKey: string;
  targetProviderId: string;
  targetBucket: string;
  targetObjectKey: string;
  contentType: string;
  contentLength: string;
  /** SHA-256 recorded on the source registry row, in lowercase hex. */
  checksumSha256Hex: string;
  status: 'pending' | 'copied' | 'failed';
  /** SHA-256 observed on the target after the copy, present only on success. */
  verifiedChecksumSha256Hex?: string | null;
  failureMessage?: string | null;
}
