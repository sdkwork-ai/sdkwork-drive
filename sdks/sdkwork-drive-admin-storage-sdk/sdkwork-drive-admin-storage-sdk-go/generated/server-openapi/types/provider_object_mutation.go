package types


type ProviderObjectMutation struct {
	ProviderId string `json:"providerId"`
	Bucket string `json:"bucket"`
	ObjectKey string `json:"objectKey"`
	Changed bool `json:"changed"`
}
