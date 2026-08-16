export interface CreateDownloadUrlResponse {
  downloadUrl: string;
  signedSourceUrl: string;
  expiresAtEpochMs: string;
  method: string;
}
