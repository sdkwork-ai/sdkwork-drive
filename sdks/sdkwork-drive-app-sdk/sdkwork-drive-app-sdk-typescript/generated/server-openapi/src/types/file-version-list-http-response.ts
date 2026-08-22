import type { FileVersionListData } from './file-version-list-data';

export interface FileVersionListHttpResponse {
  code: 0;
  data: unknown & FileVersionListData;
  /** Server-owned request correlation id. */
  traceId: string;
}
