import type { SandboxAccessLevel } from './sandbox-access-level';

export interface InitialSandboxUserGrantRequest {
  /** When true, creates an explicit grant for the current verified user. */
  enabled: boolean;
  accessLevel?: SandboxAccessLevel;
}
