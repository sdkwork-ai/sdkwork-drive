package types


type DriveWatchChannel struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	SpaceId string `json:"spaceId"`
	NodeId string `json:"nodeId"`
	ResourceType string `json:"resourceType"`
	ResourceId string `json:"resourceId"`
	ChannelType string `json:"channelType"`
	Address string `json:"address"`
	ExpirationEpochMs int `json:"expirationEpochMs"`
	LifecycleStatus string `json:"lifecycleStatus"`
	Version int `json:"version"`
}
