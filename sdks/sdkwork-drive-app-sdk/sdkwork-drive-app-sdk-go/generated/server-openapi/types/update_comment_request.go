package types


type UpdateCommentRequest struct {
	Content string `json:"content"`
	Anchor string `json:"anchor"`
	Resolved bool `json:"resolved"`
}
