import type { CreateShareLinkResponse } from './create-share-link-response';

export interface CreateShareLinkHttpResponse {
  code: 0;
  data: unknown & CreateShareLinkResponse;
  /** Server-owned request correlation id. */
  traceId: string;
}
