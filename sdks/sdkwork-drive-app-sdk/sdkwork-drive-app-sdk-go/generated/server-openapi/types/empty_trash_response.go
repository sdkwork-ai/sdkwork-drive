package types


type EmptyTrashResponse struct {
	DeletedCount string `json:"deletedCount"`
	SkippedCount string `json:"skippedCount"`
	HasMore bool `json:"hasMore"`
}
