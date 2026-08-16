export interface UploaderUploadPart {
  id: string;
  tenantId?: string;
  uploadItemId: string;
  uploadSessionId: string;
  partNo: string;
  offsetBytes: string;
  sizeBytes: string;
  etag: string;
  checksumSha256Hex?: string;
  status: string;
  retryCount: string;
  uploadedAtEpochMs?: string;
}
