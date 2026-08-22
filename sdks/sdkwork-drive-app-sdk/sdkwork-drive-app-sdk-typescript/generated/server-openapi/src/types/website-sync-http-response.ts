import type { WebsiteSyncResourceData } from './website-sync-resource-data';

export interface WebsiteSyncHttpResponse {
  code: 0;
  data: unknown & WebsiteSyncResourceData;
  /** Server-owned request correlation id. */
  traceId: string;
}
