import type { SandboxAccessLevel } from './sandbox-access-level';
import type { SandboxGrantSubjectType } from './sandbox-grant-subject-type';

export interface SandboxGrant {
  id: string;
  sandboxId: string;
  subjectType: SandboxGrantSubjectType;
  subjectId: string;
  accessLevel: SandboxAccessLevel;
  grantedBy: string;
  createdAt: string;
}
