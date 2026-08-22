export interface AssetRelation {
  id: string;
  tenantId?: string;
  assetId: string;
  relatedAssetId?: string;
  relationType: 'derived_from' | 'variant_of' | 'used_by' | 'references' | 'collection_cover' | 'external_ref';
  sourceDomain?: string;
  sourceResourceType?: string;
  sourceResourceId?: string;
  metadata?: Record<string, unknown>;
  lifecycleStatus: 'active' | 'deleted';
}
