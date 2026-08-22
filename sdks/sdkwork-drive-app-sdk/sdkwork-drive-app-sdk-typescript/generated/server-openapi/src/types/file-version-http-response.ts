import type { FileVersion } from './file-version';

export interface FileVersionHttpResponse {
  code: 0;
  data: unknown & { item: FileVersion; };
  /** Server-owned request correlation id. */
  traceId: string;
}
