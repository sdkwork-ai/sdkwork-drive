export interface CreateShareLinkResponse {
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
  /** Raw share token returned only once when the share link is created. */
  token: string;
  /** Extraction code returned only once when configured at create time. */
  accessCode?: string;
}
