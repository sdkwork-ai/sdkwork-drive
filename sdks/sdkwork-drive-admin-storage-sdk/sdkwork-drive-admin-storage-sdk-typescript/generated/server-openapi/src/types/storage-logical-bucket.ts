export interface StorageLogicalBucket {
  id: string;
  tenantId: string;
  bucketKey: string;
  displayName: string;
  objectPrefix: string;
  lifecycleStatus: 'active' | 'archived' | 'deleted';
  version: string;
  providerId?: string | null;
  providerBindingId?: string | null;
  storageRootPrefix?: string | null;
}
