package types


type Change struct {
	SequenceNo int `json:"sequenceNo"`
	TenantId string `json:"tenantId"`
	SpaceId string `json:"spaceId"`
	NodeId string `json:"nodeId"`
	EventType string `json:"eventType"`
	ActorId string `json:"actorId"`
	CreatedAt string `json:"createdAt"`
}
