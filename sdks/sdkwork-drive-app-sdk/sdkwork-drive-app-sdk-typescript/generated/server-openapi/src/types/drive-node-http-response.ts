import type { DriveNode } from './drive-node';

export interface DriveNodeHttpResponse {
  code: 0;
  data: unknown & { item: DriveNode; };
  /** Server-owned request correlation id. */
  traceId: string;
}
