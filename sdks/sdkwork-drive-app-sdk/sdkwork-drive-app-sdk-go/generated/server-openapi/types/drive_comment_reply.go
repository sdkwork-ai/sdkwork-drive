package types


type DriveCommentReply struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	NodeId string `json:"nodeId"`
	CommentId string `json:"commentId"`
	Content string `json:"content"`
	LifecycleStatus string `json:"lifecycleStatus"`
	Version int `json:"version"`
	CreatedBy string `json:"createdBy"`
	UpdatedBy string `json:"updatedBy"`
	CreatedAt string `json:"createdAt"`
	UpdatedAt string `json:"updatedAt"`
}
