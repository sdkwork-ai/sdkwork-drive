export interface DownloadPackage {
  id: string;
  tenantId: string;
  packageName: string;
  state: 'creating' | 'ready' | 'failed' | 'expired';
  storageProviderId: string;
  bucket: string;
  archiveObjectKey: string;
  contentType: 'application/zip';
  fileCount: string;
  totalBytes: string;
  archiveSizeBytes: string;
  expiresAtEpochMs: string;
  errorMessage?: string;
  createdBy: string;
  updatedBy: string;
  createdAt: string;
  updatedAt: string;
}
