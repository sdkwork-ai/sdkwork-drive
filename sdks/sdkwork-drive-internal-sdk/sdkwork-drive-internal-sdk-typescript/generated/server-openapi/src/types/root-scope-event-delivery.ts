import type { PositiveInt64String } from './positive-int64-string';

export interface RootScopeEventDelivery {
  channelId: string;
  subscriptionUuid: string;
  address: string;
  expirationEpochMs: PositiveInt64String;
  lifecycleStatus: 'ACTIVE' | 'STOPPED' | 'EXPIRED';
  version: PositiveInt64String;
  createdAt: string;
  updatedAt: string;
}
