export interface PresignedUploadPart {
  uploadUrl: string;
  expiresAtEpochMs: string;
  method: 'PUT';
  headers: Record<string, string>;
  partNo: number;
  uploadId: string;
}
