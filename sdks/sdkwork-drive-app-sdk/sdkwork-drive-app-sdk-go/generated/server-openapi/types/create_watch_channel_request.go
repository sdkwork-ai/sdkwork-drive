package types


type CreateWatchChannelRequest struct {
	Id string `json:"id"`
	SpaceId string `json:"spaceId"`
	Address string `json:"address"`
	Token string `json:"token"`
	ChannelType string `json:"channelType"`
	ExpirationEpochMs int `json:"expirationEpochMs"`
}
