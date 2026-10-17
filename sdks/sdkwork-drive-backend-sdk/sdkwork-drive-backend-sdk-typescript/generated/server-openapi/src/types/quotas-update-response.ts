export interface QuotasUpdateResponse {
  code: 0;
  /** Operation-specific payload typed per response schema. */
  data: unknown;
  /** Server-owned request correlation id. */
  traceId: string;
  tenantId: string;
  totalBytes: string;
  objectCount: string;
  quotaBytes?: string;
}
