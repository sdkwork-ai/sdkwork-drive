import type { DriveNodeProperty } from './drive-node-property';
import type { PageInfo } from './page-info';

export interface DriveNodePropertyListHttpResponse {
  code: 0;
  data: unknown & { items: DriveNodeProperty[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
