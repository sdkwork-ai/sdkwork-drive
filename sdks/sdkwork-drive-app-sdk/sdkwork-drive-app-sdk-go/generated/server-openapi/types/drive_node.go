package types


type DriveNode struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	SpaceId string `json:"spaceId"`
	ParentNodeId string `json:"parentNodeId"`
	NodeType string `json:"nodeType"`
	NodeName string `json:"nodeName"`
	LifecycleStatus string `json:"lifecycleStatus"`
	Version int `json:"version"`
	ShortcutTargetNodeId string `json:"shortcutTargetNodeId"`
	Scene string `json:"scene"`
	Source string `json:"source"`
	SpaceType string `json:"spaceType"`
	ContentState string `json:"contentState"`
	FileExtension string `json:"fileExtension"`
	ContentType string `json:"contentType"`
	ContentTypeGroup string `json:"contentTypeGroup"`
	ContentLength int `json:"contentLength"`
	FolderColor string `json:"folderColor"`
	CreatedAt string `json:"createdAt"`
	UpdatedAt string `json:"updatedAt"`
}
