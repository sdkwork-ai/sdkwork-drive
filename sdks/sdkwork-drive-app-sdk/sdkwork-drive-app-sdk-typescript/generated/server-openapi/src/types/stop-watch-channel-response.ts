import type { DriveWatchChannel } from './drive-watch-channel';

export interface StopWatchChannelResponse {
  stopped: boolean;
  channel: DriveWatchChannel;
}
