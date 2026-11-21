export interface BusinessStorageBinding {
  id: string;
  tenantId: string;
  scopeType: 'tenant' | 'space_type' | 'logical_scope';
  scopeId: string;
  logicalBucketId: string;
  logicalBucketKey: string;
  logicalBucketName: string;
  writePrefix: string;
  lifecycleStatus: 'active' | 'disabled' | 'deleted';
  version: string;
  providerId?: string | null;
  storageRootPrefix?: string | null;
}
