import type { DriveLabel } from './drive-label';

export interface LabelsCreateResponse201 {
  code: 0;
  data: unknown & { item: DriveLabel; };
  /** Server-owned request correlation id. */
  traceId: string;
}
