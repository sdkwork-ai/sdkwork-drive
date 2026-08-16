package types


type CreateFileResponse struct {
	Node DriveNode `json:"node"`
	UploadSession DriveUploadSession `json:"uploadSession"`
}
