package types


type UploadSessionMutationResponse struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	SpaceId string `json:"spaceId"`
	NodeId string `json:"nodeId"`
	Bucket string `json:"bucket"`
	ObjectKey string `json:"objectKey"`
	State string `json:"state"`
	ExpiresAtEpochMs string `json:"expiresAtEpochMs"`
	Version string `json:"version"`
	StorageProviderId string `json:"storageProviderId"`
	StorageUploadId string `json:"storageUploadId"`
}
