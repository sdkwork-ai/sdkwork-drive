import type { StorageOverview } from './storage-overview';

export interface StorageOverviewRetrieveResponse {
  code: 0;
  data: unknown & { item: StorageOverview; };
  /** Server-owned request correlation id. */
  traceId: string;
}
