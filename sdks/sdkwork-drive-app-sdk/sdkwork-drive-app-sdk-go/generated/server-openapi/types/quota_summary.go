package types


type QuotaSummary struct {
	TenantId string `json:"tenantId"`
	UsedBytes int `json:"usedBytes"`
	ObjectCount int `json:"objectCount"`
	QuotaBytes int `json:"quotaBytes"`
}
