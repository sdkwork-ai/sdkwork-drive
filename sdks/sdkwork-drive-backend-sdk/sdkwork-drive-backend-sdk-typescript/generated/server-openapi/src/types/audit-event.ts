export interface AuditEvent {
  id: string;
  tenantId: string;
  action: string;
  resourceType: string;
  resourceId: string;
  operatorId: string;
  correlationId?: string;
  traceId?: string;
  createdAt: string;
}
