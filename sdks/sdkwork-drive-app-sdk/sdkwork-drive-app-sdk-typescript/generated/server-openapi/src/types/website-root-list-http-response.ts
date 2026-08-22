import type { WebsiteRootPageData } from './website-root-page-data';

export interface WebsiteRootListHttpResponse {
  code: 0;
  data: unknown & WebsiteRootPageData;
  /** Server-owned request correlation id. */
  traceId: string;
}
