package types


type SweepResponse struct {
	ScannedCount int `json:"scannedCount"`
	AffectedCount int `json:"affectedCount"`
	DryRun bool `json:"dryRun"`
}
