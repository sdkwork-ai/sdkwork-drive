package types


type MaintenanceJob struct {
	Id string `json:"id"`
	JobType string `json:"jobType"`
	Status string `json:"status"`
	DryRun bool `json:"dryRun"`
	ScannedCount string `json:"scannedCount"`
	AffectedCount string `json:"affectedCount"`
	OperatorId string `json:"operatorId"`
	CorrelationId string `json:"correlationId"`
	TraceId string `json:"traceId"`
	ErrorMessage string `json:"errorMessage"`
	StartedAt string `json:"startedAt"`
	FinishedAt string `json:"finishedAt"`
	CreatedAt string `json:"createdAt"`
}
