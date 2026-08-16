package types


type UpdateAssetRequest struct {
	Title string `json:"title"`
	Description string `json:"description"`
	Scene string `json:"scene"`
	Source string `json:"source"`
	Tags []string `json:"tags"`
	Visibility string `json:"visibility"`
}
