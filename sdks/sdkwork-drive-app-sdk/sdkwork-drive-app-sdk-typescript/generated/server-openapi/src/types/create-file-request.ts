export interface CreateFileRequest {
  id: string;
  spaceId: string;
  parentNodeId?: string;
  nodeName: string;
  uploadSessionId: string;
  idempotencyKey: string;
  expiresAtEpochMs: string;
  /** Optional storage provider the caller wants this object written to. It must be an active provider of the caller's tenant. When omitted, the provider is resolved from the bucket, the space binding, the space-type binding, or the tenant binding, in that order. */
  storageProviderId?: string;
  bucket?: string;
  /** Deprecated compatibility field. The service ignores this value and generates an internal sdkwork-drive/v1 object key. */
  objectKey?: string;
}
