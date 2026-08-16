package types


type NodeLabel struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	NodeId string `json:"nodeId"`
	LabelId string `json:"labelId"`
	LifecycleStatus string `json:"lifecycleStatus"`
	Version int `json:"version"`
	Label DriveLabelSummary `json:"label"`
}
