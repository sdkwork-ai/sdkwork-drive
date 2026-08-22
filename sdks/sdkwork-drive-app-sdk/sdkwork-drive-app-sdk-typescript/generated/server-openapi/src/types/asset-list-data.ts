import type { AssetItem } from './asset-item';
import type { PageInfo } from './page-info';

export interface AssetListData {
  items: AssetItem[];
  pageInfo: PageInfo;
}
