import type { DriveSandboxFileContentData } from './drive-sandbox-file-content-data';

export interface DriveSandboxFileContentHttpResponse {
  code: 0;
  data: unknown & DriveSandboxFileContentData;
  /** Server-owned request correlation id. */
  traceId: string;
}
