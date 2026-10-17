import type { WebsiteRootResourceData } from './website-root-resource-data';

export interface WebsiteRootResponse {
  code: 0;
  data: unknown & WebsiteRootResourceData;
  traceId: string;
}
