package types


type AssetCollectionItem struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	CollectionId string `json:"collectionId"`
	AssetId string `json:"assetId"`
	SortOrder int `json:"sortOrder"`
}
