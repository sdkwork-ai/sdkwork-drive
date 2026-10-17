export interface ProviderObjectMultipartUpload {
  providerId: string;
  bucket: string;
  /** Drive object listing key. UTF-8 1-1024 bytes, trimmed relative key; no leading slash, double slash, NUL, or period-only path segments. */
  objectKey: string;
  /** Opaque vendor multipart upload id returned by `...multipartUpload.create`. It is never parsed, only echoed back to the vendor. */
  uploadId: string;
}
