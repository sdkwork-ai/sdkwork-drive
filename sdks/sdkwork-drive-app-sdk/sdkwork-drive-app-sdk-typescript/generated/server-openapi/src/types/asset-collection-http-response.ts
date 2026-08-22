import type { AssetCollection } from './asset-collection';

export interface AssetCollectionHttpResponse {
  code: 0;
  data: unknown & { item: AssetCollection; };
  /** Server-owned request correlation id. */
  traceId: string;
}
