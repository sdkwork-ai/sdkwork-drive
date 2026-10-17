import type { RootScopeEventDeliveryResourceData } from './root-scope-event-delivery-resource-data';

export interface RootScopeEventDeliveryResponse {
  code: 0;
  data: unknown & RootScopeEventDeliveryResourceData;
  traceId: string;
}
