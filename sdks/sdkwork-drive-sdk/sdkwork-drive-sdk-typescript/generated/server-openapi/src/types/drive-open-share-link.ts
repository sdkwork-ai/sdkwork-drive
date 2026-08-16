import type { OpenNode } from './open-node';

export interface DriveOpenShareLink {
  id: string;
  tenantId: string;
  role: string;
  expiresAtEpochMs?: string;
  downloadLimit?: string;
  downloadCount: string;
  accessCodeRequired?: boolean;
  node: OpenNode;
}
