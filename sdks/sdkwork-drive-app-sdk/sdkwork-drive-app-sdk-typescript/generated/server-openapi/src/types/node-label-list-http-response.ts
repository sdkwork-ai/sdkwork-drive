import type { NodeLabel } from './node-label';
import type { PageInfo } from './page-info';

export interface NodeLabelListHttpResponse {
  code: 0;
  data: unknown & { items: NodeLabel[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
