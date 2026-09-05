package types


type CreateFileRequest struct {
	Id string `json:"id"`
	SpaceId string `json:"spaceId"`
	ParentNodeId string `json:"parentNodeId"`
	NodeName string `json:"nodeName"`
	UploadSessionId string `json:"uploadSessionId"`
	IdempotencyKey string `json:"idempotencyKey"`
	ExpiresAtEpochMs string `json:"expiresAtEpochMs"`
	Bucket string `json:"bucket"`
	ObjectKey string `json:"objectKey"`
}
