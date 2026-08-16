package types


type SweepObjectStoreRequest struct {
	DryRun bool `json:"dryRun"`
	Limit int `json:"limit"`
}
