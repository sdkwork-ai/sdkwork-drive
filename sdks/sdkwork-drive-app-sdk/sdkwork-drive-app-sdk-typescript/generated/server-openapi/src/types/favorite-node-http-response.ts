import type { FavoriteNodeResponse } from './favorite-node-response';

export interface FavoriteNodeHttpResponse {
  code: 0;
  data: unknown & FavoriteNodeResponse;
  /** Server-owned request correlation id. */
  traceId: string;
}
