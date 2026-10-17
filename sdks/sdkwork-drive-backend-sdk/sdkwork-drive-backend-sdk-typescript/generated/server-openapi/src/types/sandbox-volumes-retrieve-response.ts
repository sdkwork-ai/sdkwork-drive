import type { SandboxVolume } from './sandbox-volume';

export interface SandboxVolumesRetrieveResponse {
  code: 0;
  data: unknown & unknown & { item: SandboxVolume; };
  /** Server-owned request correlation id. */
  traceId: string;
}
