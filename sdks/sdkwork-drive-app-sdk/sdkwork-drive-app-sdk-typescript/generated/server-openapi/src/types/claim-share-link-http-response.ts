import type { ClaimShareLinkResponse } from './claim-share-link-response';

export interface ClaimShareLinkHttpResponse {
  code: 0;
  data: unknown & ClaimShareLinkResponse;
  /** Server-owned request correlation id. */
  traceId: string;
}
