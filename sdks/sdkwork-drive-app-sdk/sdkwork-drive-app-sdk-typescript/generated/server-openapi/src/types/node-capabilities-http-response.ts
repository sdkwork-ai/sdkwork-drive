import type { NodeCapabilitiesResponse } from './node-capabilities-response';

export interface NodeCapabilitiesHttpResponse {
  code: 0;
  data: unknown & { item: NodeCapabilitiesResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
