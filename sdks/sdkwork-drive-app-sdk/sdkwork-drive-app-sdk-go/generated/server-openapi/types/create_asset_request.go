package types


type CreateAssetRequest struct {
	DriveNodeId string `json:"driveNodeId"`
	VirtualReference map[string]interface{} `json:"virtualReference"`
	Title string `json:"title"`
	Description string `json:"description"`
	Scene string `json:"scene"`
	Source string `json:"source"`
	Tags []string `json:"tags"`
}
