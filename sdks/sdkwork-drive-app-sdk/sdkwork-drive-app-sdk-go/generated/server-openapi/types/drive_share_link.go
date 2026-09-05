package types


type DriveShareLink struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	NodeId string `json:"nodeId"`
	Role string `json:"role"`
	ExpiresAtEpochMs string `json:"expiresAtEpochMs"`
	DownloadLimit string `json:"downloadLimit"`
	DownloadCount string `json:"downloadCount"`
	AccessCodeRequired bool `json:"accessCodeRequired"`
	LifecycleStatus string `json:"lifecycleStatus"`
	Version string `json:"version"`
}
