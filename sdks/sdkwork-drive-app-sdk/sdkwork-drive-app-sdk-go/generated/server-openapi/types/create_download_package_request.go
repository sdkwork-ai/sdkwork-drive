package types


type CreateDownloadPackageRequest struct {
	NodeIds []string `json:"nodeIds"`
	PackageName string `json:"packageName"`
	RequestedTtlSeconds int `json:"requestedTtlSeconds"`
}
