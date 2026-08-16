package types


type StopWatchChannelResponse struct {
	Stopped bool `json:"stopped"`
	Channel DriveWatchChannel `json:"channel"`
}
