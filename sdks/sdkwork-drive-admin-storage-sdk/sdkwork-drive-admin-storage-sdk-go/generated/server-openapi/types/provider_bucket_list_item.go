package types


type ProviderBucketListItem struct {
	Bucket string `json:"bucket"`
	Configured bool `json:"configured"`
	CreationDateEpochMs string `json:"creationDateEpochMs"`
}
