package types


type UploaderUploadPart struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	UploadItemId string `json:"uploadItemId"`
	UploadSessionId string `json:"uploadSessionId"`
	PartNo string `json:"partNo"`
	OffsetBytes string `json:"offsetBytes"`
	SizeBytes string `json:"sizeBytes"`
	Etag string `json:"etag"`
	ChecksumSha256Hex string `json:"checksumSha256Hex"`
	Status string `json:"status"`
	RetryCount string `json:"retryCount"`
	UploadedAtEpochMs string `json:"uploadedAtEpochMs"`
}
