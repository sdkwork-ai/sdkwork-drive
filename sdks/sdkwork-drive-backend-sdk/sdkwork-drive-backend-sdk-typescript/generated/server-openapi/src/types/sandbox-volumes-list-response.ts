import type { PageInfo } from './page-info';
import type { SandboxVolume } from './sandbox-volume';

export interface SandboxVolumesListResponse {
  code: 0;
  data: unknown & unknown & { items: SandboxVolume[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
