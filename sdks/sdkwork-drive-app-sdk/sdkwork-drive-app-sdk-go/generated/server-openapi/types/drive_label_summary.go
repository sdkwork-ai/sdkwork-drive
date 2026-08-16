package types


type DriveLabelSummary struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	LabelKey string `json:"labelKey"`
	DisplayName string `json:"displayName"`
	Color string `json:"color"`
	Description string `json:"description"`
	LifecycleStatus string `json:"lifecycleStatus"`
	Version int `json:"version"`
}
