export interface DriveComment {
  id: string;
  tenantId?: string;
  nodeId: string;
  content: string;
  anchor?: string;
  resolved: boolean;
  lifecycleStatus: string;
  version: string;
  createdBy: string;
  updatedBy: string;
  createdAt: string;
  updatedAt: string;
}
