import type { DriveCommentReply } from './drive-comment-reply';
import type { PageInfo } from './page-info';

export interface DriveCommentReplyListHttpResponse {
  code: 0;
  data: unknown & { items: DriveCommentReply[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
