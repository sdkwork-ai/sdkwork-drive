package types


type UpdateLabelRequest struct {
	DisplayName string `json:"displayName"`
	Color string `json:"color"`
	Description string `json:"description"`
}
