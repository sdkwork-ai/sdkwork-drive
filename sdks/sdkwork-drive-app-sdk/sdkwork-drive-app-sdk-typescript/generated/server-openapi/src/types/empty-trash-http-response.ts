import type { EmptyTrashResponse } from './empty-trash-response';

export interface EmptyTrashHttpResponse {
  code: 0;
  data: unknown & EmptyTrashResponse;
  /** Server-owned request correlation id. */
  traceId: string;
}
