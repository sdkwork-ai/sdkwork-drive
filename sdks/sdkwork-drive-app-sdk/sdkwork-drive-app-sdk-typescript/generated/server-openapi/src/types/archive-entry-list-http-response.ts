import type { ArchiveEntry } from './archive-entry';
import type { PageInfo } from './page-info';

export interface ArchiveEntryListHttpResponse {
  code: 0;
  data: unknown & { items: ArchiveEntry[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
