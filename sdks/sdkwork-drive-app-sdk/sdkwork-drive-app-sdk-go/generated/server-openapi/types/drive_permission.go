package types


type DrivePermission struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	NodeId string `json:"nodeId"`
	SubjectType string `json:"subjectType"`
	SubjectId string `json:"subjectId"`
	Role string `json:"role"`
	Inherited bool `json:"inherited"`
	LifecycleStatus string `json:"lifecycleStatus"`
	Version int `json:"version"`
}
