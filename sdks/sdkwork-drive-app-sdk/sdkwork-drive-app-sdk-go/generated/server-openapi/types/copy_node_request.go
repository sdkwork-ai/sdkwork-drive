package types


type CopyNodeRequest struct {
	Id string `json:"id"`
	TargetSpaceId string `json:"targetSpaceId"`
	TargetParentNodeId string `json:"targetParentNodeId"`
	NodeName string `json:"nodeName"`
}
