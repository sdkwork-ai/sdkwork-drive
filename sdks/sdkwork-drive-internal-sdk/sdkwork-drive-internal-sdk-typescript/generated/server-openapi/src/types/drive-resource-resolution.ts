import type { DriveResourceScopeType } from './drive-resource-scope-type';
import type { NonNegativeInt64String } from './non-negative-int64-string';
import type { NormalizedRelativePath } from './normalized-relative-path';
import type { PositiveInt64String } from './positive-int64-string';

export interface DriveResourceResolution {
  scopeType: DriveResourceScopeType;
  scopeUuid: string;
  scopeGeneration: PositiveInt64String;
  normalizedRelativePath: NormalizedRelativePath;
  resourceType: 'FILE';
  nodeId: string;
  logicalNodeVersionId: string;
  versionNo: PositiveInt64String;
  checksumSha256Hex: string;
  etag: string;
  contentType: string;
  contentLength: NonNegativeInt64String;
  lastModified: string;
  scopeStatus: 'ACTIVE';
  nodeStatus: 'ACTIVE';
  eligibility: 'ELIGIBLE';
}
