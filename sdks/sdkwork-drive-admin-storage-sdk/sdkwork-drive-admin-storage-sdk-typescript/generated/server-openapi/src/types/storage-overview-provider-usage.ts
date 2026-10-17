/** One provider row of the usage breakdown. */
export interface StorageOverviewProviderUsage {
  /** Provider configuration id. */
  providerId: string;
  /** Operator-facing provider name. */
  name: string;
  /** Provider kind key, e.g. aliyun_oss. */
  providerKind: string;
  /** Provider status: active, disabled or deleted. */
  status: string;
  /** Bucket the provider writes to. */
  bucket: string;
  /** Objects stored through this provider. */
  objectCount: string;
  /** Sum of content_length stored through this provider. */
  usedBytes: string;
  /** Active bindings that resolve to this provider. */
  bindingCount: string;
  /** True when this provider carries the tenant default binding. */
  isTenantDefault: boolean;
  /** Share of total used bytes held by this provider, 0..1. */
  capacityShare: number;
}
