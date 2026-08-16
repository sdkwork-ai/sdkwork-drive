package types


type EffectivePermission struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	TargetNodeId string `json:"targetNodeId"`
	NodeId string `json:"nodeId"`
	SubjectType string `json:"subjectType"`
	SubjectId string `json:"subjectId"`
	Role string `json:"role"`
	Inherited bool `json:"inherited"`
	InheritedFromNodeId string `json:"inheritedFromNodeId"`
	LifecycleStatus string `json:"lifecycleStatus"`
	Version int `json:"version"`
}
