export interface MarkUploaderPartUploadedRequest {
  uploadSessionId: string;
  offsetBytes: string;
  sizeBytes: string;
  etag: string;
  checksumSha256Hex?: string;
  uploadedAtEpochMs?: string;
}
