export interface DrivePermission {
  id: string;
  tenantId?: string;
  nodeId: string;
  subjectType?: string;
  subjectId?: string;
  role: string;
  inherited: boolean;
  lifecycleStatus: string;
  version: string;
}
