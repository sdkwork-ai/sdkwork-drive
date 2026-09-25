export type StorageProviderKind =
  | 'local_filesystem'
  | 's3_compatible'
  | 'google_cloud_storage'
  | 'aliyun_oss'
  | 'tencent_cos'
  | 'huawei_obs'
  | 'volcengine_tos'
  | 'custom'
  | `custom:${string}`;

export type StorageProviderHealthStatus = 'unknown' | 'healthy' | 'degraded' | 'unreachable';

/** Built-in provider kind registry row (服务商). */
export interface StorageProviderKindView {
  providerKind: string;
  displayName: string;
  enabled: boolean;
  sortOrder: number;
  version: number;
  configCount: number;
}

export interface StorageProviderView {
  id: string;
  providerKind: string;
  displayName: string;
  endpointUrl: string;
  region?: string;
  bucket: string;
  pathStyle: boolean;
  credentialRef?: string;
  /** Reference to a reusable platform service-provider account. */
  providerAccountId?: string;
  credentialConfigured: boolean;
  serverSideEncryptionMode?: string;
  defaultStorageClass?: string;
  status: string;
  version: number;
  strictTls: boolean;
  healthStatus?: StorageProviderHealthStatus;
  lastHealthCheckAt?: number;
  objectCount?: number;
  totalSizeBytes?: number;
}

export interface StorageProviderCapabilitiesView {
  providerId: string;
  providerKind: string;
  supportsMultipartUpload: boolean;
  supportsPresignedUploadPart: boolean;
  supportsPresignedDownload: boolean;
  supportsServerSideEncryption: boolean;
  supportsStorageClass: boolean;
  supportsCredentialRotation: boolean;
  supportedServerSideEncryptionModes: string[];
  supportedStorageClasses: string[];
}

export interface StorageProviderBindingView {
  id: string;
  tenantId: string;
  spaceId?: string;
  providerId: string;
  bindingScope: string;
  purpose: string;
  lifecycleStatus: string;
  version: number;
  storageRootPrefix?: string;
  storageProvider?: StorageProviderView;
}

export interface StorageProviderBucketView {
  providerId: string;
  bucket: string;
  exists: boolean;
}

export interface CreateStorageProviderInput {
  id: string;
  providerKind: StorageProviderKind;
  name: string;
  endpointUrl: string;
  region?: string;
  bucket: string;
  pathStyle?: boolean;
  credentialRef?: string;
  providerAccountId?: string;
  serverSideEncryptionMode?: string;
  defaultStorageClass?: string;
  status?: string;
  strictTls?: boolean;
}

export interface UpdateStorageProviderInput {
  name?: string;
  endpointUrl?: string;
  region?: string;
  bucket?: string;
  pathStyle?: boolean;
  credentialRef?: string;
  providerAccountId?: string;
  serverSideEncryptionMode?: string;
  defaultStorageClass?: string;
  status?: string;
  strictTls?: boolean;
}

export interface ListStorageProvidersInput {
  status?: string;
  pageSize?: number;
  pageToken?: string;
  signal?: AbortSignal;
}

export interface ListStorageProvidersPageResult {
  items: StorageProviderView[];
  nextPageToken?: string;
  hasMore: boolean;
}

export interface SetDefaultStorageProviderBindingInput {
  providerId: string;
  spaceId?: string;
  spaceType?: string;
  storageRootPrefix?: string;
  signal?: AbortSignal;
}

export interface StorageProviderBucketListItemView {
  bucket: string;
  configured: boolean;
  creationDate?: string;
}

export interface StorageProviderObjectView {
  key: string;
  sizeBytes: number;
  contentType?: string;
  etag?: string;
  lastModified?: string;
  isFolder: boolean;
}

export interface ListStorageProviderObjectsInput {
  prefix?: string;
  pageToken?: string;
  pageSize?: number;
  signal?: AbortSignal;
}

export interface ListStorageProviderObjectsResult {
  items: StorageProviderObjectView[];
  nextPageToken?: string;
  hasMore: boolean;
}

/** 对象内容读取结果：内容以 base64 返回（受后端 8 MiB 读取上限约束）。 */
export interface StorageProviderObjectContentView {
  providerId: string;
  bucket: string;
  objectKey: string;
  contentType?: string;
  sizeBytes: number;
  encoding: 'base64';
  content: string;
  checksumSha256: string;
}

export interface WriteStorageProviderObjectContentInput {
  content: string;
  encoding?: 'utf8' | 'base64';
  contentType?: string;
}

export interface CopyStorageProviderObjectInput {
  sourceObjectKey: string;
  destinationObjectKey: string;
}

export interface StorageProviderObjectMutationResult {
  providerId: string;
  bucket: string;
  objectKey: string;
  changed: boolean;
}

export interface StorageProviderMutationOptions {
  signal?: AbortSignal;
}

/**
 * Reusable service-provider account projected from the platform account
 * center (`iam_provider_account`). One account (for example one Aliyun
 * account) can back storage providers across every business; the projection
 * never carries credential material.
 */
export interface StorageProviderAccountView {
  id: string;
  /**
   * `platform` = a global default published by platform operators and readable
   * from every tenant; `tenant` = this application tenant's default; `user` =
   * a personal account owned by one end user.
   */
  scopeType: StorageProviderAccountScope;
  /** Set only for `user`-scoped accounts. */
  ownerUserId?: string;
  /**
   * Whether this account is its scope's default for the vendor + environment,
   * i.e. what a consumer picks when it does not name an account.
   */
  isDefault: boolean;
  vendorCode: string;
  accountCode: string;
  displayName: string;
  accountType: string;
  environment: string;
  externalAccountId?: string;
  capabilityCodes: string[];
  regionCode?: string;
  status: string;
  credentialConfigured: boolean;
  credentialCount: number;
  version: number;
}

/** How widely a reusable account is shared. Ordered narrowest first. */
export type StorageProviderAccountScope = 'user' | 'tenant' | 'platform';

/**
 * One built-in provider kind a bootstrap run settled.
 *
 * `accountCreated` / `credentialSeeded` are what let the page report honestly:
 * a rerun that finds everything already in place returns `false` for both, which
 * is the visible proof that the run did **not** replace keys an operator had
 * already entered.
 */
export interface StorageProviderAccountDefaultView {
  providerKind: string;
  providerId: string;
  providerCreated: boolean;
  /** Absent for a credential-free kind (`local_filesystem`). */
  vendorCode?: string;
  providerAccountId?: string;
  accountCode?: string;
  accountCreated: boolean;
  credentialSeeded: boolean;
}

export interface ListStorageProviderAccountsInput {
  vendorCode?: string;
  status?: string;
  search?: string;
  capabilityCode?: string;
  /** Restrict the list to one scope level; omit to list every visible level. */
  scopeType?: StorageProviderAccountScope;
  /** Restrict the list to one owner; only accepted when it is the calling user. */
  ownerUserId?: string;
  /** Pin the list to the calling user's own accounts. */
  mine?: boolean;
  /** Whether platform-wide accounts are included. Defaults to true. */
  includePlatform?: boolean;
  signal?: AbortSignal;
}

/** Registers a reusable account together with its access key pair. */
export interface CreateStorageProviderAccountInput {
  displayName: string;
  vendorCode: string;
  accountCode: string;
  accountType?: string;
  environment?: string;
  externalAccountId?: string;
  regionCode?: string;
  /** Defaults to `tenant` on the server. */
  scopeType?: StorageProviderAccountScope;
  /** Only meaningful with `scopeType: 'user'`; defaults to the caller. */
  ownerUserId?: string;
  /** Make this account its scope's default for the vendor + environment. */
  isDefault?: boolean;
  accessKeyId: string;
  secretAccessKey: string;
  sessionToken?: string;
}

/**
 * 存储中心仪表盘聚合。
 *
 * capacity / providers / bindings 三块都是同一租户口径（后端不含未被引用的
 * provider）；catalog（服务商目录）是平台级的，表里没有租户维度，故意不入租户口径。
 */
export interface StorageOverviewView {
  generatedAt: string;
  scopeTenantId: string;
  capacity: StorageOverviewCapacityView;
  providers: StorageOverviewProvidersView;
  bindings: StorageOverviewBindingsView;
  catalog: StorageOverviewCatalogView;
  trend: StorageOverviewTrendPointView[];
}

export interface StorageOverviewCapacityView {
  totalObjectCount: number;
  activeObjectCount: number;
  deletedObjectCount: number;
  usedBytes: number;
  averageObjectBytes: number;
  largestObjectBytes?: number;
  bucketCount: number;
  /** 租户配额上限；未配置时为 undefined。 */
  quotaBytes?: number;
  quotaConfigured: boolean;
  /** usedBytes/quotaBytes，可能大于 1（超配额）；未配置配额时缺省。 */
  quotaUsageRatio?: number;
}

export interface StorageOverviewProvidersView {
  totalCount: number;
  activeCount: number;
  disabledCount: number;
  deletedCount: number;
  usage: StorageOverviewProviderUsageView[];
}

export interface StorageOverviewProviderUsageView {
  providerId: string;
  name: string;
  providerKind: string;
  status: string;
  bucket: string;
  objectCount: number;
  usedBytes: number;
  bindingCount: number;
  isTenantDefault: boolean;
  capacityShare: number;
}

export interface StorageOverviewBindingsView {
  totalCount: number;
  activeCount: number;
  inactiveCount: number;
  byScope: StorageOverviewBindingScopeCountsView;
  hasTenantDefault: boolean;
  tenantDefaultBindingId?: string;
  tenantDefaultProviderId?: string;
}

export interface StorageOverviewBindingScopeCountsView {
  tenantCount: number;
  spaceCount: number;
  spaceTypeCount: number;
}

export interface StorageOverviewCatalogView {
  totalCount: number;
  enabledCount: number;
  disabledCount: number;
}

export interface StorageOverviewTrendPointView {
  /** 闭区间月份桶，形如 `2026-09`。 */
  periodLabel: string;
  objectCount: number;
  bytes: number;
}

export interface GetStorageOverviewInput {
  /** 趋势桶数量，服务端限定 1..24。 */
  trendMonths?: number;
  signal?: AbortSignal;
}
