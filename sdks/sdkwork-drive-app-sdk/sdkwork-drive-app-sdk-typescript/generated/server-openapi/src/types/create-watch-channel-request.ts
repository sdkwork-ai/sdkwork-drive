export interface CreateWatchChannelRequest {
  id: string;
  spaceId?: string;
  address: string;
  token?: string;
  channelType?: 'web_hook';
  expirationEpochMs: string;
}
