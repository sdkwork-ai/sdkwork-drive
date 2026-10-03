export interface ProviderBucketListItem {
  bucket: string;
  configured: boolean;
  creationDateEpochMs?: string;
  /** Region the vendor reports for this bucket on its account bucket inventory (`Location`). The inventory is an account-level read that spans regions, so the region belongs to the row rather than to the provider configuration; omitted for vendors that report none. */
  region?: string;
}
