package types


type PresignedUploadPart struct {
	UploadUrl string `json:"uploadUrl"`
	ExpiresAtEpochMs int `json:"expiresAtEpochMs"`
	Method string `json:"method"`
	Headers map[string]string `json:"headers"`
	PartNo int `json:"partNo"`
	UploadId string `json:"uploadId"`
}
