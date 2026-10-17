import type { StorageOverviewBindingScopeCounts } from './storage-overview-binding-scope-counts';

/** Binding inventory and health signals. */
export interface StorageOverviewBindings {
  /** Binding rows regardless of lifecycle status. */
  totalCount: string;
  /** Bindings in lifecycle status active. */
  activeCount: string;
  /** Bindings that are disabled or deleted. */
  inactiveCount: string;
  byScope: StorageOverviewBindingScopeCounts;
  /** False means new writes in this tenant have no default provider. */
  hasTenantDefault: boolean;
  /** Id of the tenant default binding; null when unset. */
  tenantDefaultBindingId: string | null;
  /** Provider resolved by the tenant default binding; null when unset. */
  tenantDefaultProviderId: string | null;
}
