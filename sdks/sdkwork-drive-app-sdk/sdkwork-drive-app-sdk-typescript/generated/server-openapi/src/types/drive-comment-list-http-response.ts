import type { DriveComment } from './drive-comment';
import type { PageInfo } from './page-info';

export interface DriveCommentListHttpResponse {
  code: 0;
  data: unknown & { items: DriveComment[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
