package types


type SweepUploadSessionsRequest struct {
	NowEpochMs string `json:"nowEpochMs"`
	DryRun bool `json:"dryRun"`
	Limit string `json:"limit"`
}
