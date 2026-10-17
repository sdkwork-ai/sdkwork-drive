import type { DriveResourceResolutionResourceData } from './drive-resource-resolution-resource-data';

export interface DriveResourceResolutionResponse {
  code: 0;
  data: unknown & DriveResourceResolutionResourceData;
  traceId: string;
}
