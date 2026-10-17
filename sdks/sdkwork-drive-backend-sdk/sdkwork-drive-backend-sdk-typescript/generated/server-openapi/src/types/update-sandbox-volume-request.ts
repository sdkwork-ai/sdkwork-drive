import type { SandboxAccessLevel } from './sandbox-access-level';
import type { SandboxLifecycleStatus } from './sandbox-lifecycle-status';

export interface UpdateSandboxVolumeRequest {
  displayName?: string;
  /** Admin-only provider root. local_filesystem values are canonicalized server-side. */
  providerRootRef?: string;
  lifecycleStatus?: SandboxLifecycleStatus;
  defaultAccess?: SandboxAccessLevel;
  expectedVersion: string;
}
