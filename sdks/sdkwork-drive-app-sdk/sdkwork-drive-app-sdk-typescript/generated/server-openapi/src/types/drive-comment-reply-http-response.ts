import type { DriveCommentReply } from './drive-comment-reply';

export interface DriveCommentReplyHttpResponse {
  code: 0;
  data: unknown & { item: DriveCommentReply; };
  /** Server-owned request correlation id. */
  traceId: string;
}
