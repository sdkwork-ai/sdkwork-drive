export interface StorageProviderKind {
  /** Provider kind key, e.g. aliyun_oss. */
  providerKind: string;
  /** Provider brand display name. */
  displayName: string;
  /** Whether the provider kind is enabled for new or reactivated configurations. */
  enabled: boolean;
  /** Display ordering for admin surfaces. */
  sortOrder: string;
  /** Optimistic locking version. */
  version: string;
  /** Number of non-deleted provider configurations of this kind. */
  configCount: string;
}
