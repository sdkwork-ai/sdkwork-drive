package types


type UpdateQuotaPolicyRequest struct {
	QuotaBytes string `json:"quotaBytes"`
	ClearTenantPolicy bool `json:"clearTenantPolicy"`
}
