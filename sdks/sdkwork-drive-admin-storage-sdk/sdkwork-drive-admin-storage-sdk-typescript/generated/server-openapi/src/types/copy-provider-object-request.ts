export interface CopyProviderObjectRequest {
  /** Drive object key. UTF-8 1-1024 bytes, trimmed relative key; no leading/trailing slash, double slash, NUL, or period-only path segments. */
  sourceObjectKey: string;
  /** Drive object key. UTF-8 1-1024 bytes, trimmed relative key; no leading/trailing slash, double slash, NUL, or period-only path segments. */
  destinationObjectKey: string;
  /** Source bucket override for the copy. Absent means the bucket configured on the storage provider. The provider endpoint, region, and credentials still apply. */
  sourceBucket?: string;
  /** S3-compatible bucket name. DNS-compatible 3-63 characters; lowercase letters, digits, dots, and hyphens only; must start and end with a letter or digit; no IPv4-looking names, adjacent dots, dot-hyphen adjacency, or reserved S3 affixes. */
  destinationBucket?: string;
  /** Region the copy's buckets live in, exactly as the bucket inventory reports it on that row. The server-side copy is issued at the destination endpoint, so this is what addresses the destination bucket's own regional endpoint per the vendor's own endpoint convention; absent, or a convention that cannot express it, keeps the endpoint the provider configuration stores. */
  region?: string;
  metadataDirective?: 'COPY' | 'REPLACE';
}
