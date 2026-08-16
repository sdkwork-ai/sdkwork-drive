export interface Change {
  sequenceNo: string;
  tenantId?: string;
  spaceId: string;
  nodeId?: string;
  eventType: string;
  actorId: string;
  createdAt: string;
}
