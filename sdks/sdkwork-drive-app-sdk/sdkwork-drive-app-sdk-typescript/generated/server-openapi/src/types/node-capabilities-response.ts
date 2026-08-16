export interface NodeCapabilitiesResponse {
  tenantId?: string;
  nodeId: string;
  subjectType?: 'user' | 'group';
  subjectId?: string;
  role: 'none' | 'reader' | 'commenter' | 'writer' | 'owner';
  source: 'none' | 'permission' | 'space_owner';
  permissionId: string | null;
  inherited: boolean;
  inheritedFromNodeId: string | null;
  canRead: boolean;
  canComment: boolean;
  canWrite: boolean;
  canDownload: boolean;
  canCopy: boolean;
  canMove: boolean;
  canTrash: boolean;
  canRestore: boolean;
  canDelete: boolean;
  canShare: boolean;
  canManagePermissions: boolean;
  canManageVersions: boolean;
}
