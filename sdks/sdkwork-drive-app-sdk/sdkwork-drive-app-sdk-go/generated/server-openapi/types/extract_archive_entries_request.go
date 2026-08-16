package types


type ExtractArchiveEntriesRequest struct {
	EntryPaths []string `json:"entryPaths"`
	TargetParentNodeId string `json:"targetParentNodeId"`
}
