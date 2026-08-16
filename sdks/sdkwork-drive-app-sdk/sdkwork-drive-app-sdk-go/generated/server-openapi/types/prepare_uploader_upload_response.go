package types


type PrepareUploaderUploadResponse struct {
	UploadItem UploaderUploadItem `json:"uploadItem"`
	UploadSession UploadSessionMutationResponse `json:"uploadSession"`
}
