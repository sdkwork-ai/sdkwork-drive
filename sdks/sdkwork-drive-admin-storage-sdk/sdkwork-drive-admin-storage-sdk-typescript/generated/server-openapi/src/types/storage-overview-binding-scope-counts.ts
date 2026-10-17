/** Binding counts per binding scope. */
export interface StorageOverviewBindingScopeCounts {
  /** Bindings scoped to the whole tenant. */
  tenantCount: string;
  /** Bindings scoped to a single space. */
  spaceCount: string;
  /** Bindings scoped to a space type. */
  spaceTypeCount: string;
}
