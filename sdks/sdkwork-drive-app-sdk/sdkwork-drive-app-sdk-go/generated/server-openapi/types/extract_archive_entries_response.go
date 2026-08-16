package types


type ExtractArchiveEntriesResponse struct {
	Items []DriveNode `json:"items"`
	ExtractedCount int `json:"extractedCount"`
}
