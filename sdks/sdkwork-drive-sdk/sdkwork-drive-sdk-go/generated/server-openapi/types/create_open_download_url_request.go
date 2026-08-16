package types


type CreateOpenDownloadUrlRequest struct {
	RequestedTtlSeconds int `json:"requestedTtlSeconds"`
	AccessCode string `json:"accessCode"`
}
