package types


type UploaderRetentionRequest struct {
	Mode string `json:"mode"`
	TtlSeconds string `json:"ttlSeconds"`
	CleanupAction string `json:"cleanupAction"`
	HardDeleteAfterSeconds string `json:"hardDeleteAfterSeconds"`
}
