export interface AssetCollection {
  id: string;
  tenantId?: string;
  organizationId?: string;
  userId: string;
  title: string;
  description?: string;
  collectionType?: 'manual' | 'smart' | 'system';
  visibility?: 'private' | 'organization' | 'public';
  lifecycleStatus: 'active' | 'archived' | 'deleted';
  createdAt: string;
  updatedAt: string;
}
