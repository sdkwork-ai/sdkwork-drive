package types


type UpdateShareLinkRequest struct {
	Role string `json:"role"`
	ExpiresAtEpochMs int `json:"expiresAtEpochMs"`
	DownloadLimit int `json:"downloadLimit"`
}
