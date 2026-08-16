package types


type DriveNodeProperty struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	NodeId string `json:"nodeId"`
	PropertyKey string `json:"propertyKey"`
	PropertyValue string `json:"propertyValue"`
	Visibility string `json:"visibility"`
	LifecycleStatus string `json:"lifecycleStatus"`
	Version int `json:"version"`
}
