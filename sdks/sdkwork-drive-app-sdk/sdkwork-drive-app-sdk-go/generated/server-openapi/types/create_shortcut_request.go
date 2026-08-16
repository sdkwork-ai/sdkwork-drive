package types


type CreateShortcutRequest struct {
	Id string `json:"id"`
	SpaceId string `json:"spaceId"`
	ParentNodeId string `json:"parentNodeId"`
	NodeName string `json:"nodeName"`
	TargetNodeId string `json:"targetNodeId"`
}
