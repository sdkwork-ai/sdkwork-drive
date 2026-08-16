package types


type CreateLabelRequest struct {
	Id string `json:"id"`
	LabelKey string `json:"labelKey"`
	DisplayName string `json:"displayName"`
	Color string `json:"color"`
	Description string `json:"description"`
}
