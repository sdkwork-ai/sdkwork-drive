package types


type ClaimShareLinkResponse struct {
	ShareLinkId string `json:"shareLinkId"`
	NodeId string `json:"nodeId"`
	SpaceId string `json:"spaceId"`
	Role string `json:"role"`
	PermissionId string `json:"permissionId"`
	AlreadyClaimed bool `json:"alreadyClaimed"`
}
