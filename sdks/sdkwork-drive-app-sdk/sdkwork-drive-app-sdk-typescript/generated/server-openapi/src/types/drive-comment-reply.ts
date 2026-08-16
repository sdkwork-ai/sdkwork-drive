export interface DriveCommentReply {
  id: string;
  tenantId?: string;
  nodeId: string;
  commentId: string;
  content: string;
  lifecycleStatus: string;
  version: string;
  createdBy: string;
  updatedBy: string;
  createdAt: string;
  updatedAt: string;
}
