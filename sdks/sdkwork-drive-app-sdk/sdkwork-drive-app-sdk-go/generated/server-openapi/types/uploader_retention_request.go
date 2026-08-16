package types


type UploaderRetentionRequest struct {
	Mode string `json:"mode"`
	TtlSeconds int `json:"ttlSeconds"`
	CleanupAction string `json:"cleanupAction"`
	HardDeleteAfterSeconds int `json:"hardDeleteAfterSeconds"`
}
