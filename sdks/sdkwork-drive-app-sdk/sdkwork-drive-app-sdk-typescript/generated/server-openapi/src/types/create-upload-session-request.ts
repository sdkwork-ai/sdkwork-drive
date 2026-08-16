export interface CreateUploadSessionRequest {
  sessionId: string;
  spaceId: string;
  nodeId: string;
  bucket?: string;
  /** Deprecated compatibility field. The service ignores this value and generates an internal sdkwork-drive/v1 object key. */
  objectKey?: string;
  idempotencyKey: string;
  expiresAtEpochMs: string;
}
