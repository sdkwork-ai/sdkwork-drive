package types


type OpenNode struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	SpaceId string `json:"spaceId"`
	NodeType string `json:"nodeType"`
	NodeName string `json:"nodeName"`
	ContentType string `json:"contentType"`
	ContentLength int `json:"contentLength"`
}
