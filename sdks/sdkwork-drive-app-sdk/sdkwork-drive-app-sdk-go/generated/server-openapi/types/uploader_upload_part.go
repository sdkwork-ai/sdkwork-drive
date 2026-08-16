package types


type UploaderUploadPart struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	UploadItemId string `json:"uploadItemId"`
	UploadSessionId string `json:"uploadSessionId"`
	PartNo int `json:"partNo"`
	OffsetBytes int `json:"offsetBytes"`
	SizeBytes int `json:"sizeBytes"`
	Etag string `json:"etag"`
	ChecksumSha256Hex string `json:"checksumSha256Hex"`
	Status string `json:"status"`
	RetryCount int `json:"retryCount"`
	UploadedAtEpochMs int `json:"uploadedAtEpochMs"`
}
