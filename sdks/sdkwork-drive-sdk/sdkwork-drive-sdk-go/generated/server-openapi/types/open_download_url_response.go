package types


type OpenDownloadUrlResponse struct {
	DownloadUrl string `json:"downloadUrl"`
	ExpiresAtEpochMs int `json:"expiresAtEpochMs"`
	Method string `json:"method"`
}
