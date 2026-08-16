export interface DriveOpenRequest {
  requestId: string;
  section: 'recent';
  nodeId: string;
  spaceId?: string;
  intent: 'preview';
}
