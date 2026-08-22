import type { MediaResource } from './media-resource';

export interface AssetItem {
  /** Alias of driveNodeId. */
  assetId: string;
  /** Deprecated alias of assetId. */
  id?: string;
  tenantId?: string;
  organizationId?: string;
  userId?: string;
  driveSpaceId: string;
  driveNodeId: string;
  driveUri?: string;
  nodeType: 'file' | 'virtual_reference';
  assetKind: 'file' | 'image' | 'video' | 'audio' | 'document' | 'model' | 'other';
  assetType?: string;
  title: string;
  description?: string;
  scene?: string;
  source?: string;
  sourceType?: 'upload' | 'ai_generated' | 'imported' | 'edited' | 'system';
  sourceDomain?: string;
  sourceResourceType?: string;
  sourceResourceId?: string;
  tags?: string[];
  visibility?: 'private' | 'organization' | 'public';
  lifecycleStatus: 'active' | 'trashed' | 'archived' | 'deleted';
  resourceSnapshot?: MediaResource;
  thumbnailDriveNodeId?: string;
  createdAt: string;
  updatedAt: string;
}
