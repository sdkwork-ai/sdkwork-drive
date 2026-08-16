export interface CopyProviderObjectRequest {
  /** Drive object key. UTF-8 1-1024 bytes, trimmed relative key; no leading/trailing slash, double slash, NUL, or period-only path segments. */
  sourceObjectKey: string;
  /** Drive object key. UTF-8 1-1024 bytes, trimmed relative key; no leading/trailing slash, double slash, NUL, or period-only path segments. */
  destinationObjectKey: string;
  /** S3-compatible bucket name. DNS-compatible 3-63 characters; lowercase letters, digits, dots, and hyphens only; must start and end with a letter or digit; no IPv4-looking names, adjacent dots, dot-hyphen adjacency, or reserved S3 affixes. */
  destinationBucket?: string;
  metadataDirective?: 'COPY' | 'REPLACE';
}
