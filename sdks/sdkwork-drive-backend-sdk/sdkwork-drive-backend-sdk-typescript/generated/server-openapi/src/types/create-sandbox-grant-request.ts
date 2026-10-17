import type { SandboxAccessLevel } from './sandbox-access-level';
import type { SandboxGrantSubjectType } from './sandbox-grant-subject-type';

export interface CreateSandboxGrantRequest {
  granteeType: SandboxGrantSubjectType;
  granteeId: string;
  accessLevel: SandboxAccessLevel;
}
