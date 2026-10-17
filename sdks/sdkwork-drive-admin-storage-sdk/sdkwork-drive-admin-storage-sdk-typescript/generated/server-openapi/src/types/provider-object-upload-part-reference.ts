export interface ProviderObjectUploadPartReference {
  /** Vendor part number, 1-based and contiguous. Vendors require at least 5 MiB per part except the last one. */
  partNumber: number;
  /** ETag the vendor returned for that part. Browsers can only read it when the bucket CORS policy exposes the ETag header. */
  etag: string;
}
