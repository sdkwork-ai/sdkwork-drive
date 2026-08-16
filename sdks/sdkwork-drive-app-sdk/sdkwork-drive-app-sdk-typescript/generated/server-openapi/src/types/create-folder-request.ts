export interface CreateFolderRequest {
  /** Optional client-supplied node id for offline or idempotent retries. When omitted, the service assigns a server-generated id. */
  id?: string;
  spaceId: string;
  parentNodeId?: string;
  nodeName: string;
}
