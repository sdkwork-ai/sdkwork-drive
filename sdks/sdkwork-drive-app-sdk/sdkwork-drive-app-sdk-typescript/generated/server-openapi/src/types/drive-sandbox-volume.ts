import type { DriveSandboxCapabilities } from './drive-sandbox-capabilities';

export interface DriveSandboxVolume {
  /** Opaque sandbox identifier. */
  id: string;
  displayName: string;
  /** Opaque logical root entry identifier. */
  rootEntryId: string;
  effectiveAccess: 'full' | 'read_only';
  lifecycleStatus: 'active' | 'read_only';
  capabilities: DriveSandboxCapabilities;
  /** Opaque optimistic concurrency revision. */
  revision: string;
}
