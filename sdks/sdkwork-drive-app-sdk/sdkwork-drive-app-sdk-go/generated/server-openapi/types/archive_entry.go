package types


type ArchiveEntry struct {
	Path string `json:"path"`
	Name string `json:"name"`
	IsDirectory bool `json:"isDirectory"`
	UncompressedSizeBytes string `json:"uncompressedSizeBytes"`
	CompressedSizeBytes string `json:"compressedSizeBytes"`
	ContentType string `json:"contentType"`
}
