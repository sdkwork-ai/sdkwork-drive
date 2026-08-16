package types


type EmptyTrashResponse struct {
	DeletedCount int `json:"deletedCount"`
	SkippedCount int `json:"skippedCount"`
	HasMore bool `json:"hasMore"`
}
