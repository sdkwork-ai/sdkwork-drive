export interface LabelsUpdateResponse {
  code: 0;
  /** Operation-specific payload typed per response schema. */
  data: unknown;
  /** Server-owned request correlation id. */
  traceId: string;
  id: string;
  tenantId: string;
  labelKey: string;
  displayName: string;
  color?: string | null;
  description?: string | null;
  lifecycleStatus: 'active' | 'deleted';
  version: number;
}
