import type { Change } from './change';
import type { PageInfo } from './page-info';

export interface ChangeListData {
  items: Change[];
  pageInfo: PageInfo;
}
