export interface DriveWatchChannel {
  id: string;
  tenantId?: string;
  spaceId?: string | null;
  nodeId?: string | null;
  resourceType: 'changes' | 'node';
  resourceId?: string | null;
  channelType: 'web_hook';
  address: string;
  expirationEpochMs: string;
  lifecycleStatus: 'active' | 'stopped' | 'expired';
  version: number;
}
