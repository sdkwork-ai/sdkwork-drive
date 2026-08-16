package types


type CreateDownloadUrlResponse struct {
	DownloadUrl string `json:"downloadUrl"`
	SignedSourceUrl string `json:"signedSourceUrl"`
	ExpiresAtEpochMs int `json:"expiresAtEpochMs"`
	Method string `json:"method"`
}
