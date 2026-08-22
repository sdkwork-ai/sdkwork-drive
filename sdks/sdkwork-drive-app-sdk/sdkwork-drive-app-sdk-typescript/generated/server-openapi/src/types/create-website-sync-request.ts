import type { NonNegativeInt64String } from './non-negative-int64-string';
import type { PositiveInt64String } from './positive-int64-string';

export interface CreateWebsiteSyncRequest {
  expectedRootVersion: PositiveInt64String;
  expectedGeneration: PositiveInt64String;
  manifestSha256: string;
  manifestFileCount: PositiveInt64String & unknown;
  manifestTotalBytes: NonNegativeInt64String;
  expiresAt: string;
}
