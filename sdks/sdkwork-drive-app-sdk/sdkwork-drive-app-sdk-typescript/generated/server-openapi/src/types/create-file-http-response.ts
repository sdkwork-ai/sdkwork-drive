import type { CreateFileResponse } from './create-file-response';

export interface CreateFileHttpResponse {
  code: 0;
  data: unknown & CreateFileResponse;
  /** Server-owned request correlation id. */
  traceId: string;
}
