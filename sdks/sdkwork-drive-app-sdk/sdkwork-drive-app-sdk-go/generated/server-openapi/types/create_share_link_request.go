package types


type CreateShareLinkRequest struct {
	Id string `json:"id"`
	Token string `json:"token"`
	Role string `json:"role"`
	ExpiresAtEpochMs string `json:"expiresAtEpochMs"`
	DownloadLimit string `json:"downloadLimit"`
	AccessCode string `json:"accessCode"`
}
