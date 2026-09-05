package types


type DownloadPackage struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	PackageName string `json:"packageName"`
	State string `json:"state"`
	StorageProviderId string `json:"storageProviderId"`
	Bucket string `json:"bucket"`
	ArchiveObjectKey string `json:"archiveObjectKey"`
	ContentType string `json:"contentType"`
	FileCount string `json:"fileCount"`
	TotalBytes string `json:"totalBytes"`
	ArchiveSizeBytes string `json:"archiveSizeBytes"`
	ExpiresAtEpochMs string `json:"expiresAtEpochMs"`
	ErrorMessage string `json:"errorMessage"`
	CreatedBy string `json:"createdBy"`
	UpdatedBy string `json:"updatedBy"`
	CreatedAt string `json:"createdAt"`
	UpdatedAt string `json:"updatedAt"`
}
