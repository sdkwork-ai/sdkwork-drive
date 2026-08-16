package types


type DownloadPackageResponse struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	PackageName string `json:"packageName"`
	State string `json:"state"`
	StorageProviderId string `json:"storageProviderId"`
	Bucket string `json:"bucket"`
	ArchiveObjectKey string `json:"archiveObjectKey"`
	ContentType string `json:"contentType"`
	FileCount int `json:"fileCount"`
	TotalBytes int `json:"totalBytes"`
	ArchiveSizeBytes int `json:"archiveSizeBytes"`
	ExpiresAtEpochMs int `json:"expiresAtEpochMs"`
	DownloadUrl string `json:"downloadUrl"`
	SignedSourceUrl string `json:"signedSourceUrl"`
	Method string `json:"method"`
	Items []DownloadPackageItem `json:"items"`
}
