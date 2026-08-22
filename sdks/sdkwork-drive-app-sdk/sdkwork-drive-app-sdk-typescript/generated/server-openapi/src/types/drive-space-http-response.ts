import type { DriveSpace } from './drive-space';

export interface DriveSpaceHttpResponse {
  code: 0;
  data: unknown & { item: DriveSpace; };
  /** Server-owned request correlation id. */
  traceId: string;
}
