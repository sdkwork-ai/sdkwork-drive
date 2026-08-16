package types


type DownloadPackageItem struct {
	NodeId string `json:"nodeId"`
	NodeName string `json:"nodeName"`
	ArchivePath string `json:"archivePath"`
	Bucket string `json:"bucket"`
	ObjectKey string `json:"objectKey"`
	ContentType string `json:"contentType"`
	ContentLength int `json:"contentLength"`
	ChecksumSha256Hex string `json:"checksumSha256Hex"`
}
