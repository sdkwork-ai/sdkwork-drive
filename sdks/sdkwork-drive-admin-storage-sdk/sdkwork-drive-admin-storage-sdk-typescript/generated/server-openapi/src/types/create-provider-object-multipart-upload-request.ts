export interface CreateProviderObjectMultipartUploadRequest {
  /** Drive object listing key. UTF-8 1-1024 bytes, trimmed relative key; no leading slash, double slash, NUL, or period-only path segments. */
  objectKey: string;
  /** Content type recorded on the finished object. Signed into each part grant. */
  contentType?: string | null;
  /** Optional lowercase hex SHA-256 of the whole object, when the client already has it. */
  checksumSha256Hex?: string | null;
}
