import type { DriveSandboxVolumeListData } from './drive-sandbox-volume-list-data';

export interface DriveSandboxVolumeListHttpResponse {
  code: 0;
  data: unknown & DriveSandboxVolumeListData;
  /** Server-owned request correlation id. */
  traceId: string;
}
