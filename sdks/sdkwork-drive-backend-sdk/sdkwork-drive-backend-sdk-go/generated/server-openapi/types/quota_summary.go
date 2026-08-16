package types


type QuotaSummary struct {
	TenantId string `json:"tenantId"`
	TotalBytes int `json:"totalBytes"`
	ObjectCount int `json:"objectCount"`
	QuotaBytes int `json:"quotaBytes"`
}
