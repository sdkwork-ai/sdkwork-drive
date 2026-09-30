package types


type CreateUploadSessionRequest struct {
	SessionId string `json:"sessionId"`
	SpaceId string `json:"spaceId"`
	NodeId string `json:"nodeId"`
	StorageProviderId string `json:"storageProviderId"`
	Bucket string `json:"bucket"`
	ObjectKey string `json:"objectKey"`
	IdempotencyKey string `json:"idempotencyKey"`
	ExpiresAtEpochMs string `json:"expiresAtEpochMs"`
}
