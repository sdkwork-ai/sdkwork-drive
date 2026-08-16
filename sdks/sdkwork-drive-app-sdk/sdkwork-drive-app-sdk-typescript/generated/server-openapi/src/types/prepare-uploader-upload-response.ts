import type { UploaderUploadItem } from './uploader-upload-item';
import type { UploadSessionMutationResponse } from './upload-session-mutation-response';

export interface PrepareUploaderUploadResponse {
  uploadItem: UploaderUploadItem;
  uploadSession: UploadSessionMutationResponse;
}
