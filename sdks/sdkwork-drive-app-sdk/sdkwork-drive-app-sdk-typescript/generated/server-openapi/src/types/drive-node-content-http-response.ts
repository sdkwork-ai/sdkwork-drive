import type { DriveNodeContentData } from './drive-node-content-data';

export interface DriveNodeContentHttpResponse {
  code: 0;
  data: unknown & DriveNodeContentData;
  /** Server-owned request correlation id. */
  traceId: string;
}
