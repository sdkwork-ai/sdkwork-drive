import type { StartPageTokenResponse } from './start-page-token-response';

export interface StartPageTokenHttpResponse {
  code: 0;
  data: unknown & StartPageTokenResponse;
  /** Server-owned request correlation id. */
  traceId: string;
}
