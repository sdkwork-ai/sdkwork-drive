export interface PresignProviderObjectUploadPartsRequest {
  /** Drive object listing key. UTF-8 1-1024 bytes, trimmed relative key; no leading slash, double slash, NUL, or period-only path segments. */
  objectKey: string;
  /** Opaque vendor multipart upload id returned by `...multipartUpload.create`. It is never parsed, only echoed back to the vendor. */
  uploadId: string;
  /** Part numbers to sign in this call. Batched so a single request can cover the parts a client is about to send concurrently. */
  partNumbers: number[];
  /** Grant lifetime. Defaults to 3600 seconds when absent. */
  expiresInSeconds?: number | null;
}
