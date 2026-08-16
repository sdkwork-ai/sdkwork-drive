export interface UploaderUploadItem {
  id: string;
  taskId: string;
  tenantId?: string;
  organizationId?: string;
  userId?: string;
  actorType: string;
  actorId: string;
  appId: string;
  appResourceType: string;
  appResourceId: string;
  uploadProfileCode: string;
  fileFingerprint: string;
  spaceId: string;
  nodeId: string;
  uploadSessionId?: string;
  storageProviderId?: string;
  storageUploadId?: string;
  originalFileName: string;
  fileExtension?: string;
  contentType: string;
  contentTypeGroup: string;
  detectedContentType?: string;
  contentLength: string;
  checksumSha256Hex?: string;
  chunkSizeBytes: string;
  totalParts: string;
  uploadedPartsCount: string;
  uploadedBytes: string;
  status: string;
  retentionMode: string;
  retentionExpiresAtEpochMs?: string;
  cleanupAction?: string;
  hardDeleteAfterEpochMs?: string;
  cleanupStatus: string;
  postProcessStatus: string;
  /** Drive uploader usage context identifier. Optional semantic context for idempotency, ownership, and cleanup scoping. */
  scene?: string;
  /** Drive uploader usage context identifier. Optional semantic context for idempotency, ownership, and cleanup scoping. */
  source?: string;
}
