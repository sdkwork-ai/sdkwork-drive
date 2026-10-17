import type { SandboxAccessLevel } from './sandbox-access-level';
import type { SandboxLifecycleStatus } from './sandbox-lifecycle-status';
import type { SandboxProviderKind } from './sandbox-provider-kind';

export interface SandboxVolume {
  id: string;
  tenantId: string;
  /** Owning organization resolved from the verified backend session. */
  organizationId: string;
  displayName: string;
  rootEntryId: string;
  providerKind: SandboxProviderKind;
  /** Canonical provider root visible only on the protected Drive backend-admin API. It must never appear in app/open API responses, logs, or audit payloads. */
  providerRootRef: string;
  lifecycleStatus: SandboxLifecycleStatus;
  defaultAccess: SandboxAccessLevel;
  version: string;
  createdBy: string;
  updatedBy: string;
  createdAt: string;
  updatedAt: string;
}
