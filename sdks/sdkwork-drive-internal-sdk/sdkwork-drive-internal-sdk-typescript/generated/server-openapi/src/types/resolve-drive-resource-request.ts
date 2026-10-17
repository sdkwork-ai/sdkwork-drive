import type { DriveResourceScopeType } from './drive-resource-scope-type';
import type { NormalizedRelativePath } from './normalized-relative-path';
import type { PositiveInt64String } from './positive-int64-string';

export interface ResolveDriveResourceRequest {
  scopeType: DriveResourceScopeType;
  scopeUuid: string;
  relativePath: NormalizedRelativePath;
  pinnedGeneration?: PositiveInt64String;
  pinnedNodeVersionId?: string;
}
