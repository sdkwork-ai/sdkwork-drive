export interface CreateDownloadPackageRequest {
  nodeIds: string[];
  packageName?: string;
  requestedTtlSeconds?: number;
}
