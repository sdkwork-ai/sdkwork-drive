export interface ProviderObjectContent {
  providerId: string;
  bucket: string;
  objectKey: string;
  contentType?: string;
  sizeBytes: string;
  encoding: 'base64';
  /** Base64-encoded object bytes. */
  content: string;
  /** SHA-256 hex digest of the object bytes. */
  checksumSha256: string;
}
