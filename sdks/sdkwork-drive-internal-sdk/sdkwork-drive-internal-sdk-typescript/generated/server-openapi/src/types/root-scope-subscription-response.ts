import type { RootScopeSubscriptionResourceData } from './root-scope-subscription-resource-data';

export interface RootScopeSubscriptionResponse {
  code: 0;
  data: unknown & RootScopeSubscriptionResourceData;
  traceId: string;
}
