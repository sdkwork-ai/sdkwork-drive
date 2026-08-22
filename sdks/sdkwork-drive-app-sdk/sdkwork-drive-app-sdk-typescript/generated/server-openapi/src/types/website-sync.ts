import type { NonNegativeInt64String } from './non-negative-int64-string';
import type { PositiveInt64String } from './positive-int64-string';

export interface WebsiteSync {
  id: string;
  websiteRootUuid: string;
  spaceId: string;
  expectedRootVersion: PositiveInt64String;
  expectedGeneration: PositiveInt64String;
  stagingNodeId: string;
  manifestSha256: string;
  manifestFileCount: PositiveInt64String;
  manifestTotalBytes: NonNegativeInt64String;
  uploadedFileCount: NonNegativeInt64String;
  uploadedTotalBytes: NonNegativeInt64String;
  status: 'CREATED' | 'UPLOADING' | 'READY' | 'VALIDATING' | 'ACTIVE' | 'COMPLETED' | 'FAILED' | 'ABORTED' | 'EXPIRED';
  expiresAt: string;
  validatedAt?: string;
  activatedAt?: string;
  completedAt?: string;
  errorCode?: string;
  errorSummary?: string;
  version: PositiveInt64String;
  createdAt: string;
  updatedAt: string;
}
