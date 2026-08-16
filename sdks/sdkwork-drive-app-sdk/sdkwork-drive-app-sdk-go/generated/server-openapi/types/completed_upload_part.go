package types


type CompletedUploadPart struct {
	PartNo int `json:"partNo"`
	Etag string `json:"etag"`
}
