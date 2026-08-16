package types


type StorageProviderCapabilities struct {
	ProviderId string `json:"providerId"`
	ProviderKind string `json:"providerKind"`
	SupportsMultipartUpload bool `json:"supportsMultipartUpload"`
	SupportsPresignedUploadPart bool `json:"supportsPresignedUploadPart"`
	SupportsPresignedDownload bool `json:"supportsPresignedDownload"`
	SupportsServerSideEncryption bool `json:"supportsServerSideEncryption"`
	SupportsStorageClass bool `json:"supportsStorageClass"`
	SupportsCredentialRotation bool `json:"supportsCredentialRotation"`
	SupportedServerSideEncryptionModes []string `json:"supportedServerSideEncryptionModes"`
	SupportedStorageClasses []string `json:"supportedStorageClasses"`
}
