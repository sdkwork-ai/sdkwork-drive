package types


type DriveOpenShareLink struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	Role string `json:"role"`
	ExpiresAtEpochMs string `json:"expiresAtEpochMs"`
	DownloadLimit string `json:"downloadLimit"`
	DownloadCount string `json:"downloadCount"`
	AccessCodeRequired bool `json:"accessCodeRequired"`
	Node OpenNode `json:"node"`
}
