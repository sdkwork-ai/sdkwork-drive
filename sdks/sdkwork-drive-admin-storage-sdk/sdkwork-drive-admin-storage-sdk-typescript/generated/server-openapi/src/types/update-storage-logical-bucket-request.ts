export interface UpdateStorageLogicalBucketRequest {
  displayName?: string;
  objectPrefix?: string;
  lifecycleStatus?: 'active' | 'archived';
  providerId?: string;
  storageRootPrefix?: string;
}
