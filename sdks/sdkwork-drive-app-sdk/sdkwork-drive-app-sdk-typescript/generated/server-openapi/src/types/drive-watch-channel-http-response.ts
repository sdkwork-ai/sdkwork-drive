import type { DriveWatchChannel } from './drive-watch-channel';

export interface DriveWatchChannelHttpResponse {
  code: 0;
  data: unknown & { item: DriveWatchChannel; };
  /** Server-owned request correlation id. */
  traceId: string;
}
