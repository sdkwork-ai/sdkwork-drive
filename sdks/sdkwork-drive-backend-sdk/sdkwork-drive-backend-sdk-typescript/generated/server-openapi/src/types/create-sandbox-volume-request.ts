import type { InitialSandboxUserGrantRequest } from './initial-sandbox-user-grant-request';
import type { SandboxAccessLevel } from './sandbox-access-level';
import type { SandboxProviderKind } from './sandbox-provider-kind';

export interface CreateSandboxVolumeRequest {
  displayName: string;
  providerKind?: SandboxProviderKind;
  /** Admin-only provider root. local_filesystem values must already exist, must be directories, and are canonicalized by the server before persistence. */
  providerRootRef: string;
  /** Volume configuration default only. It never grants implicit tenant, organization, or user access. */
  defaultAccess?: SandboxAccessLevel;
  /** Omitted means enabled=true and accessLevel=full. Set enabled=false to create no initial grant. */
  initialUserGrant?: InitialSandboxUserGrantRequest;
}
