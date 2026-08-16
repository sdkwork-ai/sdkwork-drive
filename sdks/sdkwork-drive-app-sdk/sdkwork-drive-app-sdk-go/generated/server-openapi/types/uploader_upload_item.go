package types


type UploaderUploadItem struct {
	Id string `json:"id"`
	TaskId string `json:"taskId"`
	TenantId string `json:"tenantId"`
	OrganizationId string `json:"organizationId"`
	UserId string `json:"userId"`
	ActorType string `json:"actorType"`
	ActorId string `json:"actorId"`
	AppId string `json:"appId"`
	AppResourceType string `json:"appResourceType"`
	AppResourceId string `json:"appResourceId"`
	UploadProfileCode string `json:"uploadProfileCode"`
	FileFingerprint string `json:"fileFingerprint"`
	SpaceId string `json:"spaceId"`
	NodeId string `json:"nodeId"`
	UploadSessionId string `json:"uploadSessionId"`
	StorageProviderId string `json:"storageProviderId"`
	StorageUploadId string `json:"storageUploadId"`
	OriginalFileName string `json:"originalFileName"`
	FileExtension string `json:"fileExtension"`
	ContentType string `json:"contentType"`
	ContentTypeGroup string `json:"contentTypeGroup"`
	DetectedContentType string `json:"detectedContentType"`
	ContentLength int `json:"contentLength"`
	ChecksumSha256Hex string `json:"checksumSha256Hex"`
	ChunkSizeBytes int `json:"chunkSizeBytes"`
	TotalParts int `json:"totalParts"`
	UploadedPartsCount int `json:"uploadedPartsCount"`
	UploadedBytes int `json:"uploadedBytes"`
	Status string `json:"status"`
	RetentionMode string `json:"retentionMode"`
	RetentionExpiresAtEpochMs int `json:"retentionExpiresAtEpochMs"`
	CleanupAction string `json:"cleanupAction"`
	HardDeleteAfterEpochMs int `json:"hardDeleteAfterEpochMs"`
	CleanupStatus string `json:"cleanupStatus"`
	PostProcessStatus string `json:"postProcessStatus"`
	Scene string `json:"scene"`
	Source string `json:"source"`
}
