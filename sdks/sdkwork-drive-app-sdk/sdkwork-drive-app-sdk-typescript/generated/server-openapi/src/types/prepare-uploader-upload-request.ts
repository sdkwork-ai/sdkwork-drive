import type { UploaderRetentionRequest } from './uploader-retention-request';

export interface PrepareUploaderUploadRequest {
  id: string;
  taskId: string;
  appResourceType: string;
  appResourceId: string;
  uploadProfileCode?: 'generic' | 'video' | 'image' | 'audio' | 'document' | 'archive' | 'text' | 'dataset' | 'attachment' | 'avatar' | 'thumbnail';
  fileFingerprint: string;
  originalFileName: string;
  contentType: string;
  contentLength: string;
  chunkSizeBytes: string;
  spaceId?: string;
  parentNodeId?: string;
  retention?: UploaderRetentionRequest;
  nowEpochMs?: string;
  /** Drive uploader usage context identifier. Optional semantic context for idempotency, ownership, and cleanup scoping. */
  scene?: string;
  /** Drive uploader usage context identifier. Optional semantic context for idempotency, ownership, and cleanup scoping. */
  source?: string;
  /** Optional Drive share token authorizing anonymous or external uploads into an explicit target folder. The raw token is accepted only on prepare requests and is never returned. */
  shareToken?: string;
}
