package types


type CreateDownloadUrlRequest struct {
	NodeId string `json:"nodeId"`
	RequestedTtlSeconds int `json:"requestedTtlSeconds"`
}
