package types


type PrepareUploaderUploadRequest struct {
	Id string `json:"id"`
	TaskId string `json:"taskId"`
	AppResourceType string `json:"appResourceType"`
	AppResourceId string `json:"appResourceId"`
	UploadProfileCode string `json:"uploadProfileCode"`
	FileFingerprint string `json:"fileFingerprint"`
	OriginalFileName string `json:"originalFileName"`
	ContentType string `json:"contentType"`
	ContentLength string `json:"contentLength"`
	ChunkSizeBytes string `json:"chunkSizeBytes"`
	SpaceId string `json:"spaceId"`
	ParentNodeId string `json:"parentNodeId"`
	Retention UploaderRetentionRequest `json:"retention"`
	NowEpochMs string `json:"nowEpochMs"`
	Scene string `json:"scene"`
	Source string `json:"source"`
	ShareToken string `json:"shareToken"`
}
