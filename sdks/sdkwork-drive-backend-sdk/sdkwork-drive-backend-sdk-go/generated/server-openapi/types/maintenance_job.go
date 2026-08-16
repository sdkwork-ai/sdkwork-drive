package types


type MaintenanceJob struct {
	Id int `json:"id"`
	JobType string `json:"jobType"`
	Status string `json:"status"`
	DryRun bool `json:"dryRun"`
	ScannedCount int `json:"scannedCount"`
	AffectedCount int `json:"affectedCount"`
	OperatorId string `json:"operatorId"`
	CorrelationId string `json:"correlationId"`
	TraceId string `json:"traceId"`
	ErrorMessage string `json:"errorMessage"`
	StartedAt string `json:"startedAt"`
	FinishedAt string `json:"finishedAt"`
	CreatedAt string `json:"createdAt"`
}
