import type { NodePathResponse } from './node-path-response';

export interface NodePathHttpResponse {
  code: 0;
  data: unknown & NodePathResponse;
  /** Server-owned request correlation id. */
  traceId: string;
}
