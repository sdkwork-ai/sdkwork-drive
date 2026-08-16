package types


type DriveSpace struct {
	Id string `json:"id"`
	TenantId string `json:"tenantId"`
	OwnerSubjectType string `json:"ownerSubjectType"`
	OwnerSubjectId string `json:"ownerSubjectId"`
	DisplayName string `json:"displayName"`
	SpaceType string `json:"spaceType"`
	LifecycleStatus string `json:"lifecycleStatus"`
	Version int `json:"version"`
}
