import type { WebsiteRootResourceData } from './website-root-resource-data';

export interface WebsiteRootHttpResponse {
  code: 0;
  data: unknown & WebsiteRootResourceData;
  /** Server-owned request correlation id. */
  traceId: string;
}
