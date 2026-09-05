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
	ContentLength string `json:"contentLength"`
	ChecksumSha256Hex string `json:"checksumSha256Hex"`
	ChunkSizeBytes string `json:"chunkSizeBytes"`
	TotalParts string `json:"totalParts"`
	UploadedPartsCount string `json:"uploadedPartsCount"`
	UploadedBytes string `json:"uploadedBytes"`
	Status string `json:"status"`
	RetentionMode string `json:"retentionMode"`
	RetentionExpiresAtEpochMs string `json:"retentionExpiresAtEpochMs"`
	CleanupAction string `json:"cleanupAction"`
	HardDeleteAfterEpochMs string `json:"hardDeleteAfterEpochMs"`
	CleanupStatus string `json:"cleanupStatus"`
	PostProcessStatus string `json:"postProcessStatus"`
	Scene string `json:"scene"`
	Source string `json:"source"`
}
