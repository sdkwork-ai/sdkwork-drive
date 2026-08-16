export interface EffectivePermission {
  id: string;
  tenantId?: string;
  targetNodeId: string;
  nodeId: string;
  subjectType?: string;
  subjectId?: string;
  role: 'reader' | 'commenter' | 'writer' | 'owner';
  inherited: boolean;
  inheritedFromNodeId: string | null;
  lifecycleStatus: string;
  version: string;
}
