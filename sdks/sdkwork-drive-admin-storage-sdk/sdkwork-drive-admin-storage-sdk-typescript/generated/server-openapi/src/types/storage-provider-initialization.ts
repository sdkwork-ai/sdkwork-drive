export interface StorageProviderInitialization {
  providerId: string;
  bucket: string;
  /** The configured bucket was created by this run. */
  bucketCreated: boolean;
  /** The bucket is known to exist after the run. */
  bucketVerified: boolean;
  /** A tenant default binding was created by this run. */
  defaultBindingCreated: boolean;
  /** The logical bucket/business binding plane was created by this run. */
  logicalPlaneCreated: boolean;
  logicalBucketCount: string;
  businessBindingCount: string;
  /** A probe object was written with the configured account. */
  objectWriteVerified: boolean;
  /** The probe object was read back with the expected bytes. */
  objectReadVerified: boolean;
  /** The probe object was deleted. */
  objectDeleteVerified: boolean;
  /** A multipart upload was created and aborted. */
  multipartVerified: boolean;
  /** Every operation Drive storage workflows depend on passed, and the logical plane is populated. */
  ready: boolean;
  /** Actionable failure detail when `ready` is false. */
  readinessError?: string;
  /** A presigned upload-part URL was signed. */
  presignedUploadVerified: boolean;
  /** A presigned download URL was signed. */
  presignedDownloadVerified: boolean;
  /** Provider the tenant logical plane currently routes to. */
  routingProviderId?: string;
  /** The logical plane and tenant fallback now route to this provider after a successful adoption. */
  routingSwitched: boolean;
}
