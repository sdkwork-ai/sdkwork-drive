import type { DriveShareLink } from './drive-share-link';
import type { PageInfo } from './page-info';

export interface ShareLinkListHttpResponse {
  code: 0;
  data: unknown & { items: DriveShareLink[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
