import type { AuditEvent } from './audit-event';
import type { PageInfo } from './page-info';

export interface AuditEventsListResponse {
  code: 0;
  data: unknown & unknown & { items: AuditEvent[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
