package types


type UpdateQuotaPolicyRequest struct {
	QuotaBytes int `json:"quotaBytes"`
	ClearTenantPolicy bool `json:"clearTenantPolicy"`
}
