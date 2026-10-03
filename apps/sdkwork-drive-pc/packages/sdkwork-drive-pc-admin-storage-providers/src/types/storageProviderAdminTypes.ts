/**
 * Every provider kind the console can offer.
 *
 * Must stay in agreement with the backend's
 * `DriveStorageProviderKind::BUILTIN` list and the OpenAPI `providerKind`
 * enum — an entry here that the server rejects produces a 422 on submit, and a
 * server kind missing here is unreachable from the picker. `custom` and
 * `custom:<vendor>` stay the escape hatch for anything not named below.
 */
export type StorageProviderKind =
  | 'local_filesystem'
  | 's3_compatible'
  | 'google_cloud_storage'
  | 'aliyun_oss'
  | 'tencent_cos'
  | 'huawei_obs'
  | 'volcengine_tos'
  // --- Mainland China vendors ---------------------------------------------
  | 'baidu_bos'
  | 'kingsoft_ks3'
  | 'qiniu_kodo'
  | 'china_mobile_ecloud'
  | 'china_telecom_eos'
  | 'china_unicom_wo'
  // --- Rest-of-world vendors ----------------------------------------------
  | 'minio'
  | 'cloudflare_r2'
  | 'backblaze_b2'
  | 'wasabi'
  | 'digitalocean_spaces'
  | 'linode_object_storage'
  | 'vultr_object_storage'
  | 'scaleway_object_storage'
  | 'oracle_cloud_storage'
  | 'ibm_cos'
  | 'alibaba_cloud_international'
  | 'tencent_cloud_international'
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

/**
 * One step of the storage resolution chain.
 *
 * The writer resolves a target in this order, so the console shows one section
 * per step and can narrow the list to exactly that step.
 */
export type StorageBindingScope = 'space' | 'space_type' | 'tenant';

export interface ListStorageProviderBindingsInput {
  /** Restrict the list to one resolution step. */
  bindingScope?: StorageBindingScope;
  spaceId?: string;
  providerId?: string;
  lifecycleStatus?: string;
  pageSize?: number;
  pageToken?: string;
  signal?: AbortSignal;
}

export interface ListStorageProviderBindingsPageResult {
  items: StorageProviderBindingView[];
  nextPageToken?: string;
  hasMore: boolean;
}

export interface StorageProviderBucketView {
  providerId: string;
  bucket: string;
  exists: boolean;
}

/** Result of the idempotent bucket initialization (storageProviders.bucket.update). */
export interface StorageProviderBucketInitializeView {
  providerId: string;
  bucket: string;
  /** True when this call created the bucket; false when it already existed. */
  changed: boolean;
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
  /**
   * Restrict the list to one provider kind, matched by the server *inside* the
   * cursor window.
   *
   * The value is a catalogued kind (`tencent_cos`, `aliyun_oss`, ...) or the
   * single word `custom`, which the server expands to the whole
   * `custom:<vendor>` family.
   */
  providerKind?: string;
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
  /**
   * 桶的创建时间，ISO 8601（服务端下发 epoch 毫秒）。
   *
   * 服务层不做本地化：同一份数据要在多种语言的控制台里显示，格式化必须发生在知道宿主
   * 语言的那一层（`formatDriveDate`），否则英文界面会混进浏览器区域的日期格式。
   */
  creationDateIso?: string;
  /**
   * 厂商为该桶报告的地域。
   *
   * 桶清单是账号级读取、跨地域，所以地域属于这一行而不是服务商配置：它既是要显示的
   * "所属地域"，也是后续读写这个桶时该用哪个端点的依据。厂商不报告时为空。
   */
  region?: string;
}

export interface StorageProviderObjectView {
  key: string;
  sizeBytes: number;
  contentType?: string;
  etag?: string;
  /** 最后修改时间，ISO 8601；格式化同样留给知道宿主语言的显示层。 */
  lastModifiedIso?: string;
  isFolder: boolean;
}

export interface ListStorageProviderObjectsInput {
  /**
   * 目标存储桶覆盖；缺省读取服务商配置里设定的存储桶。
   *
   * 管理端的桶浏览器列的是厂商账号下的全部桶，点开任意一个都要能读它的对象，
   * 所以桶随请求走，而凭证仍然来自该服务商配置。
   */
  bucket?: string;
  /** 该存储桶所在地域，语义与 {@link StorageProviderObjectScopeOptions.region} 相同。 */
  region?: string;
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

/** 开启分片上传的输入：对象 key 与（可选的）内容类型。 */
export interface CreateStorageProviderMultipartUploadInput {
  objectKey: string;
  contentType?: string;
  checksumSha256Hex?: string;
}

export interface StorageProviderMultipartUploadView {
  providerId: string;
  bucket: string;
  objectKey: string;
  /** 厂商的不透明 uploadId：本服务只回传，不解析。 */
  uploadId: string;
}

export interface PresignStorageProviderUploadPartsInput {
  objectKey: string;
  uploadId: string;
  /** 本次要签发的分片号（1 起，最大 10000）；一批最多 100 个。 */
  partNumbers: number[];
  expiresInSeconds?: number;
}

export interface StorageProviderUploadPartGrantView {
  partNumber: number;
  /** 对 `url` 使用的方法（厂商返回 PUT）。 */
  method: string;
  /** 厂商的预签名地址：字节直传这里，不经过管理端 API。 */
  url: string;
  /**
   * 签名的一部分：必须原样随上传发送。
   *
   * 内容类型、服务端加密相关的头都在里面，少一个或改一个都会被厂商拒绝（403）。
   */
  headers: Record<string, string>;
  /** 授权到期时间（epoch 毫秒）；缺省表示厂商未返回，界面不必展示。 */
  expiresAtEpochMs?: number;
}

export interface StorageProviderUploadPartGrantsView {
  providerId: string;
  bucket: string;
  objectKey: string;
  uploadId: string;
  parts: StorageProviderUploadPartGrantView[];
}

/** 完成分片上传：提交厂商返回的 (分片号, ETag) 列表。 */
export interface CompleteStorageProviderMultipartUploadInput {
  objectKey: string;
  uploadId: string;
  parts: { partNumber: number; etag: string }[];
}

export interface AbortStorageProviderMultipartUploadInput {
  objectKey: string;
  uploadId: string;
}

export interface CopyStorageProviderObjectInput {  sourceObjectKey: string;
  destinationObjectKey: string;
  /** 源桶覆盖；缺省为服务商配置里设定的存储桶。 */
  sourceBucket?: string;
  /** 目标桶覆盖；缺省为源桶，再缺省为服务商配置里设定的存储桶。 */
  destinationBucket?: string;
  /**
   * 这些桶所在地域（桶清单里那一行的 `region`）。
   *
   * 服务端复制是在目标端点发起的，所以它决定本次请求落到哪个地域的端点；桶内改名/移动
   * 与同地域跨桶复制都由它覆盖。
   */
  region?: string;
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
 * 单对象操作的请求选项。
 *
 * 桶随请求走（`bucket`），凭证与端点仍来自服务商配置：管理端的桶浏览器管理的是
 * 厂商账号下真实存在的桶，而不是配置里写的那一个。
 */
export interface StorageProviderObjectScopeOptions extends StorageProviderMutationOptions {
  bucket?: string;
  /**
   * 被访问存储桶所在地域，取自桶清单里那一行的 `region`。
   *
   * 桶清单跨地域，点开的桶未必在服务商配置写的地域里；带上它，服务端才会按厂商自己的
   * 端点规范落到这个桶所在地域（服务端推不出该厂商规范时仍用配置端点）。
   */
  region?: string;
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
/**
 * The vendor's own credential-field vocabulary, as returned by the bootstrap.
 *
 * The bootstrap writes the placeholder key pair itself, so it also knows how
 * that vendor names the two halves. Carrying it back with the response keeps the
 * console from maintaining a second copy of the same table — the copy is what
 * drifts.
 */
export interface StorageProviderVendorCredentialFields {
  accessKeyLabel: string;
  secretKeyLabel: string;
  defaultEnvAccessKey: string;
  defaultEnvSecretKey: string;
  consoleUrl: string;
}

/**
 * The encryption modes and storage classes a vendor accepts, as returned by the
 * bootstrap.
 *
 * Same "one table, not two" rule as `StorageProviderVendorCredentialFields`: the
 * server's contract layer is the authority for what a vendor accepts, so the
 * editor's dropdowns render these values instead of a second hand-maintained
 * list in the console. `[0]` is the vendor's default, matching what the
 * bootstrap writes into a freshly created provider.
 */
export interface StorageProviderVendorCapabilityDefaults {
  serverSideEncryptionModes: string[];
  storageClasses: string[];
}

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
  /** Absent for a credential-free kind, which has no key pair to label. */
  credentialFields?: StorageProviderVendorCredentialFields;
  /** Absent for a credential-free kind, which exposes neither control. */
  vendorCapabilities?: StorageProviderVendorCapabilityDefaults;
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
  /** Rows per page; the account center defaults to 20 and caps at 200. */
  pageSize?: number;
  /** Opaque continuation from the previous page's `nextPageToken`. */
  pageToken?: string;
  signal?: AbortSignal;
}

/**
 * One page of account-center accounts plus the continuation to read the rest.
 *
 * The account list is a paginated endpoint (`PAGINATION_SPEC.md` §8): a view
 * that keeps only `items` cannot tell a vendor whose accounts start on a later
 * page from a vendor that has none, so every consumer of the list has to carry
 * the cursor. `hasMore` is the server's answer, never inferred from a short
 * page — a short page is also what a fully-read last page looks like.
 */
export interface ListStorageProviderAccountsPageResult {
  items: StorageProviderAccountView[];
  nextPageToken?: string;
  hasMore: boolean;
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
