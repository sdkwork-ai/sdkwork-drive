package types


type CreateFolderRequest struct {
	Id string `json:"id"`
	SpaceId string `json:"spaceId"`
	ParentNodeId string `json:"parentNodeId"`
	NodeName string `json:"nodeName"`
}
