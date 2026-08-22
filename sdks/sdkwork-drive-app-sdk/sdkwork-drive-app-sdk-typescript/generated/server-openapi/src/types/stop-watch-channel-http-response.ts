import type { StopWatchChannelResponse } from './stop-watch-channel-response';

export interface StopWatchChannelHttpResponse {
  code: 0;
  data: unknown & StopWatchChannelResponse;
  /** Server-owned request correlation id. */
  traceId: string;
}
