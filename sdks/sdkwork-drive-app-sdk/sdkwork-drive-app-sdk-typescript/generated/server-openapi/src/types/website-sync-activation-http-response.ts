import type { WebsiteSyncActivationResourceData } from './website-sync-activation-resource-data';

export interface WebsiteSyncActivationHttpResponse {
  code: 0;
  data: unknown & WebsiteSyncActivationResourceData;
  /** Server-owned request correlation id. */
  traceId: string;
}
