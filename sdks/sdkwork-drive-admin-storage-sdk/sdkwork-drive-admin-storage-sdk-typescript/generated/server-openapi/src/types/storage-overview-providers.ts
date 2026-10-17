import type { StorageOverviewProviderUsage } from './storage-overview-provider-usage';

/** Tenant-scoped provider inventory: only providers this tenant actually depends on are counted, i.e. providers referenced by one of the tenant's bindings or holding one of its objects. */
export interface StorageOverviewProviders {
  /** Provider configurations regardless of status. */
  totalCount: string;
  /** Providers in status active. */
  activeCount: string;
  /** Providers in status disabled. */
  disabledCount: string;
  /** Providers in status deleted. */
  deletedCount: string;
  /** Per-provider usage ordered by descending usedBytes. */
  usage: StorageOverviewProviderUsage[];
}
