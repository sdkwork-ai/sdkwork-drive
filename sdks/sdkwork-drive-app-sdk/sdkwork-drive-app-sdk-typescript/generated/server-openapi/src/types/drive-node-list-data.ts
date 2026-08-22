import type { DriveNode } from './drive-node';
import type { PageInfo } from './page-info';

export interface DriveNodeListData {
  items: DriveNode[];
  pageInfo: PageInfo;
  /** True when ACL pagination scan budget was exhausted before the requested page could be filled. */
  incompletePage?: boolean;
}
