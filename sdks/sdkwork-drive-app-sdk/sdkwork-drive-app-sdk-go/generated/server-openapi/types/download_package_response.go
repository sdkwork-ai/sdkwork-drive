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
	FileCount string `json:"fileCount"`
	TotalBytes string `json:"totalBytes"`
	ArchiveSizeBytes string `json:"archiveSizeBytes"`
	ExpiresAtEpochMs string `json:"expiresAtEpochMs"`
	DownloadUrl string `json:"downloadUrl"`
	SignedSourceUrl string `json:"signedSourceUrl"`
	Method string `json:"method"`
	Items []DownloadPackageItem `json:"items"`
}
