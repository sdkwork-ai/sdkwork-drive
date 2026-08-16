package types


type ProviderBucket struct {
	ProviderId string `json:"providerId"`
	Bucket string `json:"bucket"`
	Exists bool `json:"exists"`
}
