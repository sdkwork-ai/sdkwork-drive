package types


type DriveComment struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	NodeId string `json:"nodeId"`
	Content string `json:"content"`
	Anchor string `json:"anchor"`
	Resolved bool `json:"resolved"`
	LifecycleStatus string `json:"lifecycleStatus"`
	Version int `json:"version"`
	CreatedBy string `json:"createdBy"`
	UpdatedBy string `json:"updatedBy"`
	CreatedAt string `json:"createdAt"`
	UpdatedAt string `json:"updatedAt"`
}
