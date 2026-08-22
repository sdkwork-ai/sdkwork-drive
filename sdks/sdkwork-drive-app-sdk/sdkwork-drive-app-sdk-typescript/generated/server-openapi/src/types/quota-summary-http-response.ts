import type { QuotaSummary } from './quota-summary';

export interface QuotaSummaryHttpResponse {
  code: 0;
  data: unknown & { item: QuotaSummary; };
  /** Server-owned request correlation id. */
  traceId: string;
}
