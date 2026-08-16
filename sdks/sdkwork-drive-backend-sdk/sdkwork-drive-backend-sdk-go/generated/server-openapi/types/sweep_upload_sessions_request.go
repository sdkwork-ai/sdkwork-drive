package types


type SweepUploadSessionsRequest struct {
	NowEpochMs int `json:"nowEpochMs"`
	DryRun bool `json:"dryRun"`
	Limit int `json:"limit"`
}
