package types


type AssetCollection struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	OrganizationId string `json:"organizationId"`
	UserId string `json:"userId"`
	Title string `json:"title"`
	Description string `json:"description"`
	CollectionType string `json:"collectionType"`
	Visibility string `json:"visibility"`
	LifecycleStatus string `json:"lifecycleStatus"`
	CreatedAt string `json:"createdAt"`
	UpdatedAt string `json:"updatedAt"`
}
