package types


type FileVersion struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	NodeId string `json:"nodeId"`
	StorageObjectId string `json:"storageObjectId"`
	VersionNo string `json:"versionNo"`
	ContentType string `json:"contentType"`
	ContentLength string `json:"contentLength"`
	ChecksumSha256Hex string `json:"checksumSha256Hex"`
	LifecycleStatus string `json:"lifecycleStatus"`
	CreatedAt string `json:"createdAt"`
}
