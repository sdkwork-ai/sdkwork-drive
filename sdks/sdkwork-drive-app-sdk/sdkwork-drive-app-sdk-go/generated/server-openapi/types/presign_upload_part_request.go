package types


type PresignUploadPartRequest struct {
	UploadId string `json:"uploadId"`
	RequestedTtlSeconds int `json:"requestedTtlSeconds"`
}
