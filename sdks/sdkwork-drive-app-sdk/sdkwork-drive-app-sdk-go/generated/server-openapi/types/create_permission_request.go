package types


type CreatePermissionRequest struct {
	Id string `json:"id"`
	SubjectType string `json:"subjectType"`
	SubjectId string `json:"subjectId"`
	Role string `json:"role"`
}
