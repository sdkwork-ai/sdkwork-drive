export interface CreateStorageMigrationRequest {
  /** Caller-owned id. Supplying it makes a retried plan idempotent instead of opening a second run for the same intended migration. */
  id: string;
  name: string;
  /** Provider to drain. May be disabled; a deleted provider is rejected. */
  sourceProviderId: string;
  /** Provider to copy onto. Must be active. */
  targetProviderId: string;
  /** Target bucket. Defaults to the target provider's own bucket. */
  targetBucket?: string | null;
  /** Re-point objects and bindings at the target once every object is copied and verified. */
  applyBindingSwitch?: boolean;
}
