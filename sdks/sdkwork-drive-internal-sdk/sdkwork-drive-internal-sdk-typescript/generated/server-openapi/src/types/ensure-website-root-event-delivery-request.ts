import type { PositiveInt64String } from './positive-int64-string';

export interface EnsureWebsiteRootEventDeliveryRequest {
  address: string;
  verificationToken: string;
  expirationEpochMs: PositiveInt64String;
}
