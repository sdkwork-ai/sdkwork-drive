package types


type UpdateSpaceRequest struct {
	DisplayName string `json:"displayName"`
	PresentationIcon string `json:"presentationIcon"`
	PresentationColor string `json:"presentationColor"`
	Description string `json:"description"`
}
