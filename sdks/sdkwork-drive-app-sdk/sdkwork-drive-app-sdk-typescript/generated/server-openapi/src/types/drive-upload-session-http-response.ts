import type { DriveUploadSession } from './drive-upload-session';

export interface DriveUploadSessionHttpResponse {
  code: 0;
  data: unknown & { item: DriveUploadSession; };
  /** Server-owned request correlation id. */
  traceId: string;
}
