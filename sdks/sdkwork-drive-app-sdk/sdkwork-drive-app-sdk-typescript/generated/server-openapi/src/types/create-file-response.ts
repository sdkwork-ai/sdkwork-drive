import type { DriveNode } from './drive-node';
import type { DriveUploadSession } from './drive-upload-session';

export interface CreateFileResponse {
  node: DriveNode;
  uploadSession: DriveUploadSession;
}
