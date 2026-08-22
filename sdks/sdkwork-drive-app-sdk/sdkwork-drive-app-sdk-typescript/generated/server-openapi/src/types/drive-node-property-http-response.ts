import type { DriveNodeProperty } from './drive-node-property';

export interface DriveNodePropertyHttpResponse {
  code: 0;
  data: unknown & { item: DriveNodeProperty; };
  /** Server-owned request correlation id. */
  traceId: string;
}
