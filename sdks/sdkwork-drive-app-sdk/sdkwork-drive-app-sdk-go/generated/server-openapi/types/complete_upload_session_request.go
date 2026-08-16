package types


type CompleteUploadSessionRequest struct {
	UploadId string `json:"uploadId"`
	ContentType string `json:"contentType"`
	ContentLength int `json:"contentLength"`
	ChecksumSha256Hex string `json:"checksumSha256Hex"`
	Parts []CompletedUploadPart `json:"parts"`
}
