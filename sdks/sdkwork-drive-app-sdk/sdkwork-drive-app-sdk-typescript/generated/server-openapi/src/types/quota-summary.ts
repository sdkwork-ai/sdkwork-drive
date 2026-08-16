export interface QuotaSummary {
  tenantId?: string;
  usedBytes: string;
  objectCount: string;
  /** Configured tenant storage quota cap from SDKWORK_DRIVE_TENANT_QUOTA_MAX_BYTES when set. */
  quotaBytes?: string;
}
