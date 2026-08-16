package types


type CreateShareLinkRequest struct {
	Id string `json:"id"`
	Token string `json:"token"`
	Role string `json:"role"`
	ExpiresAtEpochMs int `json:"expiresAtEpochMs"`
	DownloadLimit int `json:"downloadLimit"`
	AccessCode string `json:"accessCode"`
}
