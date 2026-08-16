package types


type StorageProvider struct {
	Id string `json:"id"`
	ProviderKind string `json:"providerKind"`
	Name string `json:"name"`
	EndpointUrl string `json:"endpointUrl"`
	Region string `json:"region"`
	Bucket string `json:"bucket"`
	PathStyle bool `json:"pathStyle"`
	CredentialRef string `json:"credentialRef"`
	ServerSideEncryptionMode string `json:"serverSideEncryptionMode"`
	DefaultStorageClass string `json:"defaultStorageClass"`
	Status string `json:"status"`
	Version int `json:"version"`
	CredentialConfigured bool `json:"credentialConfigured"`
	StrictTls bool `json:"strictTls"`
}
