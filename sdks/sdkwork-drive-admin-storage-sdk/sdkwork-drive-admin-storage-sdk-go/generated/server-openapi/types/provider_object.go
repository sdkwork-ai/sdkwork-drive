package types


type ProviderObject struct {
	ProviderId string `json:"providerId"`
	Bucket string `json:"bucket"`
	ObjectKind string `json:"objectKind"`
	ObjectKey string `json:"objectKey"`
	ContentLength int `json:"contentLength"`
	ContentType string `json:"contentType"`
	Etag string `json:"etag"`
	VersionId string `json:"versionId"`
	StorageClass string `json:"storageClass"`
	LastModifiedEpochMs int `json:"lastModifiedEpochMs"`
}
