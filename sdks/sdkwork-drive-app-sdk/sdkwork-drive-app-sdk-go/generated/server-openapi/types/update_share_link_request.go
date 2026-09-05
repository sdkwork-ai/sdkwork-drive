package types


type UpdateShareLinkRequest struct {
	Role string `json:"role"`
	ExpiresAtEpochMs string `json:"expiresAtEpochMs"`
	DownloadLimit string `json:"downloadLimit"`
}
