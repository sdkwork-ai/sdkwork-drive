import type { DriveOpenShareLink } from './drive-open-share-link';

export interface OpenShareLinksRetrieveResponse {
  code: 0;
  data: unknown & unknown & { item: DriveOpenShareLink; };
  /** Server-owned request correlation id. */
  traceId: string;
}
