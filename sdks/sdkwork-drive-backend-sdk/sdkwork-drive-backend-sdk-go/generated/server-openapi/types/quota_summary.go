package types


type QuotaSummary struct {
	TenantId string `json:"tenantId"`
	TotalBytes string `json:"totalBytes"`
	ObjectCount string `json:"objectCount"`
	QuotaBytes string `json:"quotaBytes"`
}
