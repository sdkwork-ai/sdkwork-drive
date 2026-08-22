import type { DriveWatchChannel } from './drive-watch-channel';
import type { PageInfo } from './page-info';

export interface DriveWatchChannelListData {
  items: DriveWatchChannel[];
  pageInfo: PageInfo;
}
