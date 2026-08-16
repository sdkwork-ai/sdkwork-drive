package types


type MarkUploaderPartUploadedRequest struct {
	UploadSessionId string `json:"uploadSessionId"`
	OffsetBytes int `json:"offsetBytes"`
	SizeBytes int `json:"sizeBytes"`
	Etag string `json:"etag"`
	ChecksumSha256Hex string `json:"checksumSha256Hex"`
	UploadedAtEpochMs int `json:"uploadedAtEpochMs"`
}
