/** Tenant-level capacity figures derived from dr_drive_storage_object. */
export interface StorageOverviewCapacity {
  /** Object rows regardless of lifecycle status. */
  totalObjectCount: string;
  /** Objects currently in lifecycle status active. */
  activeObjectCount: string;
  /** Objects awaiting physical reclaim. */
  deletedObjectCount: string;
  /** Sum of content_length over active objects. */
  usedBytes: string;
  /** Mean content_length over active objects; 0 when there are none. */
  averageObjectBytes: string;
  /** Largest active object content_length; null when there are none. */
  largestObjectBytes: string | null;
  /** Distinct buckets that hold object rows in this scope. */
  bucketCount: string;
  /** Tenant quota max_bytes; null when the tenant has no quota row. */
  quotaBytes: string | null;
  /** True when a quota row exists with a non-null max_bytes. */
  quotaConfigured: boolean;
  /** usedBytes / quotaBytes. May exceed 1 when the quota is overrun; null without a quota. */
  quotaUsageRatio: number | null;
}
