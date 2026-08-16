export interface UpdateAssetRequest {
  title?: string;
  description?: string;
  scene?: string;
  source?: string;
  tags?: string[];
  visibility?: 'private' | 'organization' | 'public';
}
