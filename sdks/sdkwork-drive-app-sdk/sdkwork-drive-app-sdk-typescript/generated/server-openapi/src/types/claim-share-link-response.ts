export interface ClaimShareLinkResponse {
  shareLinkId: string;
  nodeId: string;
  spaceId: string;
  role: string;
  permissionId: string;
  alreadyClaimed: boolean;
}
