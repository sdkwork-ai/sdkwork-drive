import type { DownloadPackageItem } from './download-package-item';

export interface DownloadPackageResponse {
  id: string;
  tenantId?: string;
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
  downloadUrl: string;
  signedSourceUrl: string;
  method: 'GET';
  items: DownloadPackageItem[];
}
