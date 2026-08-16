package types


type AssetItem struct {
	AssetId string `json:"assetId"`
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	OrganizationId string `json:"organizationId"`
	UserId string `json:"userId"`
	DriveSpaceId string `json:"driveSpaceId"`
	DriveNodeId string `json:"driveNodeId"`
	DriveUri string `json:"driveUri"`
	NodeType string `json:"nodeType"`
	AssetKind string `json:"assetKind"`
	AssetType string `json:"assetType"`
	Title string `json:"title"`
	Description string `json:"description"`
	Scene string `json:"scene"`
	Source string `json:"source"`
	SourceType string `json:"sourceType"`
	SourceDomain string `json:"sourceDomain"`
	SourceResourceType string `json:"sourceResourceType"`
	SourceResourceId string `json:"sourceResourceId"`
	Tags []string `json:"tags"`
	Visibility string `json:"visibility"`
	LifecycleStatus string `json:"lifecycleStatus"`
	ResourceSnapshot MediaResource `json:"resourceSnapshot"`
	ThumbnailDriveNodeId string `json:"thumbnailDriveNodeId"`
	CreatedAt string `json:"createdAt"`
	UpdatedAt string `json:"updatedAt"`
}
