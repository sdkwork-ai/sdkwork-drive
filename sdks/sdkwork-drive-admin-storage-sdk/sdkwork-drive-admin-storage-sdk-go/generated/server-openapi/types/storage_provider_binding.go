package types


type StorageProviderBinding struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	SpaceId string `json:"spaceId"`
	ProviderId string `json:"providerId"`
	BindingScope string `json:"bindingScope"`
	Purpose string `json:"purpose"`
	LifecycleStatus string `json:"lifecycleStatus"`
	Version int `json:"version"`
	StorageProvider StorageProvider `json:"storageProvider"`
	StorageRootPrefix string `json:"storageRootPrefix"`
}
