export interface ProviderObjectUploadPartGrant {
  /** Vendor part number, 1-based and contiguous. Vendors require at least 5 MiB per part except the last one. */
  partNumber: number;
  /** HTTP method to use against `url`. Vendors return PUT. */
  method: string;
  /** Absolute presigned vendor URL. The bytes go here, never through this API. */
  url: string;
  /** Headers that are part of the signature. Send them verbatim with the upload or the vendor rejects it with 403. */
  headers?: Record<string, string>;
  expiresAtEpochMs: string | null;
}
