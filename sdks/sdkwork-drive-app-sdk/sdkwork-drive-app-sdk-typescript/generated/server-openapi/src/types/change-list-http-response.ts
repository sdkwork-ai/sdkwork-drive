import type { ChangeListData } from './change-list-data';

export interface ChangeListHttpResponse {
  code: 0;
  data: unknown & ChangeListData;
  /** Server-owned request correlation id. */
  traceId: string;
}
