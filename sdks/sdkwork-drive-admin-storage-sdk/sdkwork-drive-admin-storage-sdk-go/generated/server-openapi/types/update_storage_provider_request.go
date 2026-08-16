package types


type UpdateStorageProviderRequest struct {
	Name string `json:"name"`
	EndpointUrl string `json:"endpointUrl"`
	Region string `json:"region"`
	Bucket string `json:"bucket"`
	PathStyle bool `json:"pathStyle"`
	CredentialRef string `json:"credentialRef"`
	ServerSideEncryptionMode string `json:"serverSideEncryptionMode"`
	DefaultStorageClass string `json:"defaultStorageClass"`
	Status string `json:"status"`
	StrictTls bool `json:"strictTls"`
}
