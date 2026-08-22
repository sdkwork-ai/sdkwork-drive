import type { NonNegativeInt64String } from './non-negative-int64-string';
import type { PositiveInt64String } from './positive-int64-string';

export interface WebsiteGeneration {
  generation: PositiveInt64String;
  rootNodeId: string;
  manifestSha256?: string;
  fileCount: NonNegativeInt64String;
  totalBytes: NonNegativeInt64String;
  status: 'CURRENT' | 'RETAINED' | 'EXPIRED' | 'DELETING' | 'DELETED' | 'INVALID';
  activatedAt: string;
}
