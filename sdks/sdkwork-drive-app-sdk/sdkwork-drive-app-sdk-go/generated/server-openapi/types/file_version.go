package types


type FileVersion struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	NodeId string `json:"nodeId"`
	StorageObjectId string `json:"storageObjectId"`
	VersionNo int `json:"versionNo"`
	ContentType string `json:"contentType"`
	ContentLength int `json:"contentLength"`
	ChecksumSha256Hex string `json:"checksumSha256Hex"`
	LifecycleStatus string `json:"lifecycleStatus"`
	CreatedAt string `json:"createdAt"`
}
