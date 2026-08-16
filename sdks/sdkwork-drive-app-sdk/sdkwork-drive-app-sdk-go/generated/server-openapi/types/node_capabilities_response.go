package types


type NodeCapabilitiesResponse struct {
	TenantId string `json:"tenantId"`
	NodeId string `json:"nodeId"`
	SubjectType string `json:"subjectType"`
	SubjectId string `json:"subjectId"`
	Role string `json:"role"`
	Source string `json:"source"`
	PermissionId string `json:"permissionId"`
	Inherited bool `json:"inherited"`
	InheritedFromNodeId string `json:"inheritedFromNodeId"`
	CanRead bool `json:"canRead"`
	CanComment bool `json:"canComment"`
	CanWrite bool `json:"canWrite"`
	CanDownload bool `json:"canDownload"`
	CanCopy bool `json:"canCopy"`
	CanMove bool `json:"canMove"`
	CanTrash bool `json:"canTrash"`
	CanRestore bool `json:"canRestore"`
	CanDelete bool `json:"canDelete"`
	CanShare bool `json:"canShare"`
	CanManagePermissions bool `json:"canManagePermissions"`
	CanManageVersions bool `json:"canManageVersions"`
}
