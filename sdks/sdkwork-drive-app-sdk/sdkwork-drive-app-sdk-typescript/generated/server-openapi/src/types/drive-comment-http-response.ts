import type { DriveComment } from './drive-comment';

export interface DriveCommentHttpResponse {
  code: 0;
  data: unknown & { item: DriveComment; };
  /** Server-owned request correlation id. */
  traceId: string;
}
