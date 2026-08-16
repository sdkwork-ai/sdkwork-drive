package types


type CreateSpaceRequest struct {
	Id string `json:"id"`
	OwnerSubjectType string `json:"ownerSubjectType"`
	OwnerSubjectId string `json:"ownerSubjectId"`
	DisplayName string `json:"displayName"`
	SpaceType string `json:"spaceType"`
	PresentationIcon string `json:"presentationIcon"`
	PresentationColor string `json:"presentationColor"`
	Description string `json:"description"`
}
