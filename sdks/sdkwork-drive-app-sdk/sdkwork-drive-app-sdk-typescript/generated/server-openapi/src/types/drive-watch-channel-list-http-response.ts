import type { DriveWatchChannelListData } from './drive-watch-channel-list-data';

export interface DriveWatchChannelListHttpResponse {
  code: 0;
  data: unknown & DriveWatchChannelListData;
  /** Server-owned request correlation id. */
  traceId: string;
}
