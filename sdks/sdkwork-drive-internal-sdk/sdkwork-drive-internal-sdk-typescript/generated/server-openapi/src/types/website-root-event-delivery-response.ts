import type { WebsiteRootEventDeliveryResourceData } from './website-root-event-delivery-resource-data';

export interface WebsiteRootEventDeliveryResponse {
  code: 0;
  data: unknown & WebsiteRootEventDeliveryResourceData;
  traceId: string;
}
