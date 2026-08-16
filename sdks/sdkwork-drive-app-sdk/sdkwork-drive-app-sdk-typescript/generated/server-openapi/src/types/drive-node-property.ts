export interface DriveNodeProperty {
  id: string;
  tenantId?: string;
  nodeId: string;
  propertyKey: string;
  propertyValue: string;
  visibility: 'private' | 'app_public';
  lifecycleStatus: 'active';
  version: number;
}
