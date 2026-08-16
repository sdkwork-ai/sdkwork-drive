package types


type MediaResource struct {
	Id string `json:"id"`
	Kind string `json:"kind"`
	Source string `json:"source"`
	Url string `json:"url"`
	PublicUrl string `json:"publicUrl"`
	Uri string `json:"uri"`
	ObjectBlobId string `json:"objectBlobId"`
	FileName string `json:"fileName"`
	MimeType string `json:"mimeType"`
	SizeBytes string `json:"sizeBytes"`
	Checksum map[string]interface{} `json:"checksum"`
	Width int `json:"width"`
	Height int `json:"height"`
	DurationSeconds float64 `json:"durationSeconds"`
	AltText string `json:"altText"`
	Title string `json:"title"`
	Poster MediaResource `json:"poster"`
	Thumbnails []MediaResource `json:"thumbnails"`
	Variants []MediaResource `json:"variants"`
	Access map[string]interface{} `json:"access"`
	Ai map[string]interface{} `json:"ai"`
	Metadata map[string]interface{} `json:"metadata"`
	MediaResourceId string `json:"mediaResourceId"`
	MediaType string `json:"mediaType"`
	ContentType string `json:"contentType"`
	DurationMs int `json:"durationMs"`
	ChecksumSha256 string `json:"checksumSha256"`
}
