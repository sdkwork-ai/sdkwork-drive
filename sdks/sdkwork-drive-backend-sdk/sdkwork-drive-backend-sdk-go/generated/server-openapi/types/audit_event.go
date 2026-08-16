package types


type AuditEvent struct {
	Id int `json:"id"`
	TenantId string `json:"tenantId"`
	Action string `json:"action"`
	ResourceType string `json:"resourceType"`
	ResourceId string `json:"resourceId"`
	OperatorId string `json:"operatorId"`
	CorrelationId string `json:"correlationId"`
	TraceId string `json:"traceId"`
	CreatedAt string `json:"createdAt"`
}
