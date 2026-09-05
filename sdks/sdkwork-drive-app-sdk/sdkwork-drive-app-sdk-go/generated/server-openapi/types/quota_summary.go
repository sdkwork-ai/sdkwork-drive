package types


type QuotaSummary struct {
	TenantId string `json:"tenantId"`
	UsedBytes string `json:"usedBytes"`
	ObjectCount string `json:"objectCount"`
	QuotaBytes string `json:"quotaBytes"`
}
