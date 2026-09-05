package types


type ExtractArchiveEntriesResponse struct {
	Items []DriveNode `json:"items"`
	ExtractedCount string `json:"extractedCount"`
}
