package types


type UpdateNodeRequest struct {
	NodeName string `json:"nodeName"`
	ParentNodeId string `json:"parentNodeId"`
}
