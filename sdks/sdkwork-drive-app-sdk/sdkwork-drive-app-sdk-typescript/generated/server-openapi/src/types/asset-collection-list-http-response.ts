import type { AssetCollectionListData } from './asset-collection-list-data';

export interface AssetCollectionListHttpResponse {
  code: 0;
  data: unknown & AssetCollectionListData;
  /** Server-owned request correlation id. */
  traceId: string;
}
