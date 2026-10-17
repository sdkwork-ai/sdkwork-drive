import type { StorageOverviewBindings } from './storage-overview-bindings';
import type { StorageOverviewCapacity } from './storage-overview-capacity';
import type { StorageOverviewCatalog } from './storage-overview-catalog';
import type { StorageOverviewProviders } from './storage-overview-providers';
import type { StorageOverviewTrendPoint } from './storage-overview-trend-point';

/** Storage center dashboard aggregate. Capacity, usage and binding figures are computed over the same tenant scope so they are mutually consistent; the catalog block is platform-wide and intentionally not tenant-scoped. */
export interface StorageOverview {
  /** Server timestamp the aggregate was computed at. */
  generatedAt: string;
  /** Tenant every capacity, usage and binding figure is scoped to. The catalog block is platform-wide and ignores this. */
  scopeTenantId: string;
  capacity: StorageOverviewCapacity;
  providers: StorageOverviewProviders;
  bindings: StorageOverviewBindings;
  catalog: StorageOverviewCatalog;
  /** Monthly ingestion buckets, oldest first. */
  trend: StorageOverviewTrendPoint[];
}
