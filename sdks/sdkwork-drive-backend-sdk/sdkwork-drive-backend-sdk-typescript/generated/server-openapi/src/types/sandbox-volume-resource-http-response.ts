import type { SandboxVolume } from './sandbox-volume';

export interface SandboxVolumeResourceHttpResponse {
  code: 0;
  data: unknown & { item: SandboxVolume; };
  /** Server-owned request correlation id. */
  traceId: string;
}
