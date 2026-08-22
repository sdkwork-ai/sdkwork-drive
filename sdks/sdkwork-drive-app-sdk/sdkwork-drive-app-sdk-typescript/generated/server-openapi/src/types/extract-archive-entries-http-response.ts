import type { ExtractArchiveEntriesResponse } from './extract-archive-entries-response';

export interface ExtractArchiveEntriesHttpResponse {
  code: 0;
  data: unknown & ExtractArchiveEntriesResponse;
  /** Server-owned request correlation id. */
  traceId: string;
}
