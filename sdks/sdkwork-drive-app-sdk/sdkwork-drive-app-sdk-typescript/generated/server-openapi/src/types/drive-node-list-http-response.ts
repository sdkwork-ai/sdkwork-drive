import type { DriveNodeListData } from './drive-node-list-data';

export interface DriveNodeListHttpResponse {
  code: 0;
  data: unknown & DriveNodeListData;
  /** Server-owned request correlation id. */
  traceId: string;
}
