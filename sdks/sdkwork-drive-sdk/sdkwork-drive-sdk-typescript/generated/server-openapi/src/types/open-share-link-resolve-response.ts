import type { DriveOpenShareLink } from './drive-open-share-link';

export interface OpenShareLinkResolveResponse {
  code: 0;
  data: unknown & { item: DriveOpenShareLink; };
  /** Server-owned request correlation id. */
  traceId: string;
}
