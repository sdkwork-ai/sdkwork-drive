export interface DriveShareLink {
  id: string;
  tenantId?: string;
  nodeId: string;
  role: string;
  expiresAtEpochMs?: string;
  downloadLimit?: string;
  downloadCount: string;
  accessCodeRequired?: boolean;
  lifecycleStatus: string;
  version: string;
}
