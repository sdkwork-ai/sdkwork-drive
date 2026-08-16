package types


type CreateShareLinkResponse struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	NodeId string `json:"nodeId"`
	Role string `json:"role"`
	ExpiresAtEpochMs int `json:"expiresAtEpochMs"`
	DownloadLimit int `json:"downloadLimit"`
	DownloadCount int `json:"downloadCount"`
	AccessCodeRequired bool `json:"accessCodeRequired"`
	LifecycleStatus string `json:"lifecycleStatus"`
	Version int `json:"version"`
	Token string `json:"token"`
	AccessCode string `json:"accessCode"`
}
