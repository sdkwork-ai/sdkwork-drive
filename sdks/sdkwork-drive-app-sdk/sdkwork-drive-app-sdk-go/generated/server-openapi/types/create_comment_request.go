package types


type CreateCommentRequest struct {
	Id string `json:"id"`
	Content string `json:"content"`
	Anchor string `json:"anchor"`
}
