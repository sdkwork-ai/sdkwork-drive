package types


type CreateUploadSessionRequest struct {
	SessionId string `json:"sessionId"`
	SpaceId string `json:"spaceId"`
	NodeId string `json:"nodeId"`
	Bucket string `json:"bucket"`
	ObjectKey string `json:"objectKey"`
	IdempotencyKey string `json:"idempotencyKey"`
	ExpiresAtEpochMs int `json:"expiresAtEpochMs"`
}
