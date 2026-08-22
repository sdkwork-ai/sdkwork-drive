import type { DrivePermission } from './drive-permission';

export interface DrivePermissionHttpResponse {
  code: 0;
  data: unknown & { item: DrivePermission; };
  /** Server-owned request correlation id. */
  traceId: string;
}
