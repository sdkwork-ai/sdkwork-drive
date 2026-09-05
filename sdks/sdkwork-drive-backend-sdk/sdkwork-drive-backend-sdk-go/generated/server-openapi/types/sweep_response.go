package types


type SweepResponse struct {
	ScannedCount string `json:"scannedCount"`
	AffectedCount string `json:"affectedCount"`
	DryRun bool `json:"dryRun"`
}
