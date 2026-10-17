/** Platform-wide provider catalog (dr_drive_storage_provider_kind) coverage. The catalog has no tenant dimension, so these counts are shared by every tenant. */
export interface StorageOverviewCatalog {
  /** Catalog entries. */
  totalCount: string;
  /** Catalog entries operators may configure. */
  enabledCount: string;
  /** Catalog entries disabled for new configurations. */
  disabledCount: string;
}
