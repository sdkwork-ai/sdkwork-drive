package types


type AssetRelation struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	AssetId string `json:"assetId"`
	RelatedAssetId string `json:"relatedAssetId"`
	RelationType string `json:"relationType"`
	SourceDomain string `json:"sourceDomain"`
	SourceResourceType string `json:"sourceResourceType"`
	SourceResourceId string `json:"sourceResourceId"`
	Metadata map[string]interface{} `json:"metadata"`
	LifecycleStatus string `json:"lifecycleStatus"`
}
