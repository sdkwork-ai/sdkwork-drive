package types


type ProviderBucketMutation struct {
	ProviderId string `json:"providerId"`
	Bucket string `json:"bucket"`
	Changed bool `json:"changed"`
}
