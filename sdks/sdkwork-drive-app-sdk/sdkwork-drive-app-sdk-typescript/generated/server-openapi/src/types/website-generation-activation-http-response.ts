import type { WebsiteGenerationActivationResourceData } from './website-generation-activation-resource-data';

export interface WebsiteGenerationActivationHttpResponse {
  code: 0;
  data: unknown & WebsiteGenerationActivationResourceData;
  /** Server-owned request correlation id. */
  traceId: string;
}
