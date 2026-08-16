export interface FileVersion {
  id: string;
  tenantId?: string;
  nodeId: string;
  storageObjectId?: string;
  versionNo: string;
  contentType: string;
  contentLength: string;
  checksumSha256Hex: string;
  lifecycleStatus: string;
  createdAt: string;
}
