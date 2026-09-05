package types


type SweepObjectStoreRequest struct {
	DryRun bool `json:"dryRun"`
	Limit string `json:"limit"`
}
