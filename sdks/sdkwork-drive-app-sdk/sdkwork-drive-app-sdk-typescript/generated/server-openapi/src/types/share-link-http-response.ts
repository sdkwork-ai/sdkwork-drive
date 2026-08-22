import type { DriveShareLink } from './drive-share-link';

export interface ShareLinkHttpResponse {
  code: 0;
  data: unknown & { item: DriveShareLink; };
  /** Server-owned request correlation id. */
  traceId: string;
}
