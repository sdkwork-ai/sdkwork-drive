import type { PageInfo } from './page-info';
import type { SandboxVolume } from './sandbox-volume';

export interface SandboxVolumeListHttpResponse {
  code: 0;
  data: unknown & { items: SandboxVolume[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
