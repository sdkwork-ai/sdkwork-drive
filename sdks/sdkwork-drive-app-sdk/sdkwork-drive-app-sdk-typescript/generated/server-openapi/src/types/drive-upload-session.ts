export interface DriveUploadSession {
  id: string;
  tenantId?: string;
  spaceId: string;
  nodeId: string;
  bucket: string;
  objectKey: string;
  idempotencyKey: string;
  state: 'created' | 'uploading' | 'completing' | 'completed' | 'aborted' | 'expired';
  expiresAtEpochMs: string;
  version: string;
  /** Drive storage provider id bound to this upload session. */
  storageProviderId: string;
  /** Provider-side multipart upload id used by the configured object store. */
  storageUploadId: string;
}
