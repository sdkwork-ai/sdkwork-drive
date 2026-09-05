package types


type MarkUploaderPartUploadedRequest struct {
	UploadSessionId string `json:"uploadSessionId"`
	OffsetBytes string `json:"offsetBytes"`
	SizeBytes string `json:"sizeBytes"`
	Etag string `json:"etag"`
	ChecksumSha256Hex string `json:"checksumSha256Hex"`
	UploadedAtEpochMs string `json:"uploadedAtEpochMs"`
}
