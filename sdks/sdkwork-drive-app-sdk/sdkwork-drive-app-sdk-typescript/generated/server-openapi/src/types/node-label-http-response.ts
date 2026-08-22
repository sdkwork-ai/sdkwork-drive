import type { NodeLabel } from './node-label';

export interface NodeLabelHttpResponse {
  code: 0;
  data: unknown & { item: NodeLabel; };
  /** Server-owned request correlation id. */
  traceId: string;
}
