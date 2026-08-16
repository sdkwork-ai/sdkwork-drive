export interface CreateShortcutRequest {
  id: string;
  spaceId: string;
  parentNodeId?: string;
  nodeName: string;
  targetNodeId: string;
}
