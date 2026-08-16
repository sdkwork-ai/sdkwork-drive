package types


type ArchiveEntry struct {
	Path string `json:"path"`
	Name string `json:"name"`
	IsDirectory bool `json:"isDirectory"`
	UncompressedSizeBytes int `json:"uncompressedSizeBytes"`
	CompressedSizeBytes int `json:"compressedSizeBytes"`
	ContentType string `json:"contentType"`
}
