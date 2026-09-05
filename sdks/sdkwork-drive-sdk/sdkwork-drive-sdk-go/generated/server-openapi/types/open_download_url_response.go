package types


type OpenDownloadUrlResponse struct {
	DownloadUrl string `json:"downloadUrl"`
	ExpiresAtEpochMs string `json:"expiresAtEpochMs"`
	Method string `json:"method"`
}
