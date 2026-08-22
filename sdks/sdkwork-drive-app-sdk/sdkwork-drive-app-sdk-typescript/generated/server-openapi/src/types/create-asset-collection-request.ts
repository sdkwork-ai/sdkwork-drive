export interface CreateAssetCollectionRequest {
  title: string;
  description?: string;
  collectionType?: 'manual' | 'smart' | 'system';
  visibility?: 'private' | 'organization' | 'public';
}
