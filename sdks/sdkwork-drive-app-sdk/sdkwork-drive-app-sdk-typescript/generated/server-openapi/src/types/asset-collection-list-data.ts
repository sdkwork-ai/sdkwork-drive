import type { AssetCollection } from './asset-collection';
import type { PageInfo } from './page-info';

export interface AssetCollectionListData {
  items: AssetCollection[];
  pageInfo: PageInfo;
}
