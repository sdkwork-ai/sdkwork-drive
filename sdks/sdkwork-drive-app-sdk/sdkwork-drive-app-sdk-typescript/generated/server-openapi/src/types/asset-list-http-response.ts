import type { AssetListData } from './asset-list-data';

export interface AssetListHttpResponse {
  code: 0;
  data: unknown & AssetListData;
  /** Server-owned request correlation id. */
  traceId: string;
}
