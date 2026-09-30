use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateStorageProviderRequest {
    pub(crate) id: String,
    pub(crate) provider_kind: String,
    pub(crate) name: String,
    pub(crate) endpoint_url: String,
    pub(crate) region: Option<String>,
    pub(crate) bucket: String,
    pub(crate) path_style: Option<bool>,
    pub(crate) strict_tls: Option<bool>,
    pub(crate) credential_ref: Option<String>,
    pub(crate) provider_account_id: Option<String>,
    pub(crate) server_side_encryption_mode: Option<String>,
    pub(crate) default_storage_class: Option<String>,
    pub(crate) status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UpdateStorageProviderRequest {
    pub(crate) name: Option<String>,
    pub(crate) endpoint_url: Option<String>,
    pub(crate) region: Option<String>,
    pub(crate) bucket: Option<String>,
    pub(crate) path_style: Option<bool>,
    pub(crate) strict_tls: Option<bool>,
    pub(crate) credential_ref: Option<String>,
    pub(crate) provider_account_id: Option<String>,
    pub(crate) server_side_encryption_mode: Option<String>,
    pub(crate) default_storage_class: Option<String>,
    pub(crate) status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RotateStorageProviderCredentialRequest {
    pub(crate) credential_ref: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SetStorageProviderKindEnabledRequest {
    pub(crate) enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListStorageProvidersQuery {
    /// Stored provider kind, or `custom` for the whole `custom:<vendor>` family.
    ///
    /// Declared on the list operation itself so the filter composes with the
    /// cursor window: filtering a page after it was fetched cannot answer for a
    /// kind whose row sits on a later page.
    ///
    /// Spelled `provider_kind` on the wire, like `page_size` and `cursor`:
    /// multi-word query parameters are lower_snake_case (`API_SPEC.md` §13), and
    /// a new filter must not add another camelCase alias.
    #[serde(rename = "provider_kind")]
    pub(crate) provider_kind: Option<String>,
    pub(crate) status: Option<String>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub(crate) page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DefaultStorageProviderBindingQuery {
    pub(crate) space_id: Option<String>,
    pub(crate) space_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListStorageProviderBindingsQuery {
    pub(crate) space_id: Option<String>,
    pub(crate) provider_id: Option<String>,
    pub(crate) lifecycle_status: Option<String>,
    /// Restrict the list to one resolution step (`tenant`, `space`, `space_type`).
    ///
    /// A console that renders one section per step has to read that step's rows
    /// as a set: the unfiltered list is ordered space → space type → tenant, so a
    /// tenant with enough space-scoped bindings pushes the space-type rows past
    /// the first page and the page then renders them as unbound.
    ///
    /// Spelled `binding_scope` on the wire, like `page_size` and `cursor`:
    /// multi-word query parameters are lower_snake_case (`API_SPEC.md` §13).
    #[serde(rename = "binding_scope")]
    pub(crate) binding_scope: Option<String>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub(crate) page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct DeleteDefaultStorageProviderBindingQuery {
    pub(crate) space_id: Option<String>,
    pub(crate) space_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SetDefaultStorageProviderBindingRequest {
    pub(crate) space_id: Option<String>,
    pub(crate) space_type: Option<String>,
    pub(crate) provider_id: String,
    pub(crate) storage_root_prefix: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListProviderBucketsQuery {
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub(crate) page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListProviderObjectsQuery {
    pub(crate) prefix: Option<String>,
    pub(crate) delimiter: Option<String>,
    #[serde(rename = "cursor")]
    pub(crate) page_token: Option<String>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<u16>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CopyProviderObjectRequest {
    pub(crate) source_object_key: String,
    pub(crate) destination_object_key: String,
    pub(crate) destination_bucket: Option<String>,
    pub(crate) metadata_directive: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageProviderResponse {
    pub(crate) id: String,
    pub(crate) provider_kind: String,
    pub(crate) name: String,
    pub(crate) endpoint_url: String,
    pub(crate) region: Option<String>,
    pub(crate) bucket: String,
    pub(crate) path_style: bool,
    pub(crate) strict_tls: bool,
    pub(crate) credential_ref: Option<String>,
    pub(crate) provider_account_id: Option<String>,
    pub(crate) server_side_encryption_mode: Option<String>,
    pub(crate) default_storage_class: Option<String>,
    pub(crate) status: String,
    pub(crate) version: i64,
    pub(crate) credential_configured: bool,
}

/// One built-in provider kind a bootstrap run settled.
///
/// The two booleans are the point of the row: `account_created` and
/// `credential_seeded` distinguish "this run filled the gap" from "something was
/// already here and was deliberately left alone", which is what an operator
/// needs in order to trust that re-running the bootstrap cannot have replaced
/// keys they had already entered.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageProviderAccountDefaultResponse {
    pub(crate) provider_kind: String,
    pub(crate) provider_id: String,
    pub(crate) provider_created: bool,
    /// Absent for a credential-free kind (`local_filesystem`).
    pub(crate) vendor_code: Option<String>,
    /// The account-center account the provider is bound to; absent for a
    /// credential-free kind.
    pub(crate) provider_account_id: Option<String>,
    pub(crate) account_code: Option<String>,
    pub(crate) account_created: bool,
    pub(crate) credential_seeded: bool,
    /// The vendor's own credential-field vocabulary, so the console labels the
    /// key pair the way that vendor does (`SecretId`/`SecretKey` for Tencent,
    /// `AK`/`SK` for Huawei, …). Absent for a credential-free kind, which has no
    /// key pair to label.
    ///
    /// The bootstrap knows this because it also writes the placeholder pair; a
    /// second hand-maintained table in the console would drift from it.
    pub(crate) credential_fields: Option<VendorCredentialFieldResponse>,
    /// The encryption modes and storage classes this vendor accepts, in the
    /// contract's preference order, so the console's dropdowns offer the same
    /// values the server would accept instead of a second hand-maintained list.
    ///
    /// Absent for a credential-free kind (`local_filesystem`), which exposes
    /// neither control. An empty vector means "this vendor offers no choice",
    /// which the console renders as a hidden control rather than a fabricated
    /// tier the store would reject.
    pub(crate) vendor_capabilities: Option<VendorCapabilityDefaultsResponse>,
}

/// Per-vendor capability vocabulary carried by the bootstrap response.
///
/// This is the wire form of the contract's `supported_sse_modes()` /
/// `supported_storage_classes()`; it exists so the console's editor dropdowns
/// and the server's `capabilities` handler read one table instead of two.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VendorCapabilityDefaultsResponse {
    /// Accepted `x-cos-server-side-encryption` (and S3 equivalents), `[0]` being
    /// the default a fresh configuration should carry.
    pub(crate) server_side_encryption_modes: Vec<String>,
    /// Accepted storage classes, `[0]` being the vendor's standard tier.
    pub(crate) storage_classes: Vec<String>,
}

/// Vendor credential-field labels carried by the bootstrap response.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VendorCredentialFieldResponse {
    /// Label for the public half of the key pair.
    pub(crate) access_key_label: String,
    /// Label for the private half of the key pair.
    pub(crate) secret_key_label: String,
    /// Vendor-namespaced environment variables an operator may prefer over a
    /// literal pair, mirrored from the console's own defaults.
    pub(crate) default_env_access_key: String,
    pub(crate) default_env_secret_key: String,
    /// Console deep link where the operator mints the key pair.
    pub(crate) console_url: String,
}

/// Reusable service-provider account projected for the storage admin console.
///
/// This is a read-only reference view of the platform account center row
/// (`iam_provider_account`); the storage plane never exposes credential
/// material, only whether an active credential exists.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageProviderAccountResponse {
    pub(crate) id: String,
    /// `platform` | `tenant` | `user`.
    pub(crate) scope_type: String,
    /// Present only for `user`-scoped accounts.
    pub(crate) owner_user_id: Option<String>,
    /// Whether this account is its scope's default for the vendor +
    /// environment, i.e. what a consumer gets without naming an account.
    pub(crate) is_default: bool,
    pub(crate) vendor_code: String,
    pub(crate) account_code: String,
    pub(crate) display_name: String,
    pub(crate) account_type: String,
    pub(crate) environment: String,
    pub(crate) external_account_id: Option<String>,
    pub(crate) capability_codes: Vec<String>,
    pub(crate) region_code: Option<String>,
    pub(crate) status: String,
    pub(crate) credential_configured: bool,
    pub(crate) credential_count: i64,
    pub(crate) version: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListStorageProviderAccountsQuery {
    pub(crate) vendor_code: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) search: Option<String>,
    pub(crate) capability_code: Option<String>,
    /// Narrow the list to one scope: `platform` | `tenant` | `user`.
    pub(crate) scope_type: Option<String>,
    /// Narrow the list to one owner. Only honoured together with
    /// `scopeType=user`, and only when the caller owns it.
    pub(crate) owner_user_id: Option<String>,
    /// Convenience switch for "my own accounts": pins the list to the caller's
    /// personal `user` scope without having to know its own user id.
    pub(crate) mine: Option<bool>,
    /// Whether platform-wide accounts are included. Defaults to `true` so the
    /// console shows the global defaults a tenant can reuse.
    pub(crate) include_platform: Option<bool>,
    #[serde(rename = "page_size")]
    pub(crate) page_size: Option<i64>,
    #[serde(rename = "cursor")]
    pub(crate) page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateStorageProviderAccountRequest {
    pub(crate) display_name: String,
    pub(crate) vendor_code: String,
    pub(crate) account_code: String,
    pub(crate) account_type: Option<String>,
    pub(crate) environment: Option<String>,
    pub(crate) external_account_id: Option<String>,
    pub(crate) region_code: Option<String>,
    /// `platform` (platform operators only) | `tenant` (default) | `user`.
    pub(crate) scope_type: Option<String>,
    /// Only meaningful with `scopeType=user`; defaults to the caller. Naming
    /// somebody else's user id is rejected.
    pub(crate) owner_user_id: Option<String>,
    /// Make this account its scope's default for the vendor + environment.
    pub(crate) is_default: Option<bool>,
    pub(crate) access_key_id: String,
    pub(crate) secret_access_key: String,
    pub(crate) session_token: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageProviderCapabilitiesResponse {
    pub(crate) provider_id: String,
    pub(crate) provider_kind: String,
    pub(crate) supports_multipart_upload: bool,
    pub(crate) supports_presigned_upload_part: bool,
    pub(crate) supports_presigned_download: bool,
    pub(crate) supports_server_side_encryption: bool,
    pub(crate) supports_storage_class: bool,
    pub(crate) supports_credential_rotation: bool,
    pub(crate) supported_server_side_encryption_modes: Vec<String>,
    pub(crate) supported_storage_classes: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageProviderKindResponse {
    pub(crate) provider_kind: String,
    pub(crate) display_name: String,
    pub(crate) enabled: bool,
    pub(crate) sort_order: i64,
    pub(crate) version: i64,
    pub(crate) config_count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TestStorageProviderResponse {
    pub(crate) provider_id: String,
    pub(crate) reachable: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderBucketResponse {
    pub(crate) provider_id: String,
    pub(crate) bucket: String,
    pub(crate) exists: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderBucketListItemResponse {
    pub(crate) bucket: String,
    pub(crate) configured: bool,
    pub(crate) creation_date_epoch_ms: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderBucketMutationResponse {
    pub(crate) provider_id: String,
    pub(crate) bucket: String,
    pub(crate) changed: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderObjectResponse {
    pub(crate) provider_id: String,
    pub(crate) bucket: String,
    pub(crate) object_kind: String,
    pub(crate) object_key: String,
    pub(crate) content_length: u64,
    pub(crate) content_type: Option<String>,
    pub(crate) etag: Option<String>,
    pub(crate) version_id: Option<String>,
    pub(crate) storage_class: Option<String>,
    pub(crate) last_modified_epoch_ms: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderObjectMutationResponse {
    pub(crate) provider_id: String,
    pub(crate) bucket: String,
    pub(crate) object_key: String,
    pub(crate) changed: bool,
}

/// 对象内容读取响应：内容以 base64 传输（任意字节安全），配套大小与校验和。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderObjectContentResponse {
    pub(crate) provider_id: String,
    pub(crate) bucket: String,
    pub(crate) object_key: String,
    pub(crate) content_type: Option<String>,
    pub(crate) size_bytes: u64,
    pub(crate) encoding: String,
    pub(crate) content: String,
    pub(crate) checksum_sha256: String,
}

/// 对象内容写入请求：`encoding` 为 `utf8`（默认）或 `base64`；
/// `object_key` 以 `/` 结尾且内容为空时创建目录占位对象。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UpdateProviderObjectContentRequest {
    pub(crate) content: String,
    pub(crate) encoding: Option<String>,
    pub(crate) content_type: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageProviderBindingResponse {
    pub(crate) id: String,
    pub(crate) tenant_id: String,
    pub(crate) space_id: Option<String>,
    pub(crate) provider_id: String,
    pub(crate) binding_scope: String,
    pub(crate) purpose: String,
    pub(crate) storage_root_prefix: String,
    pub(crate) lifecycle_status: String,
    pub(crate) version: i64,
    pub(crate) storage_provider: StorageProviderResponse,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct OffsetPage {
    pub(crate) limit: i64,
    pub(crate) offset: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageOverviewQuery {
    /// Number of monthly trend buckets. Clamped by the handler to 1..24.
    pub(crate) trend_months: Option<i64>,
}

/// Storage center dashboard aggregate.
///
/// Capacity, usage and binding figures are tenant-scoped; the catalog block is
/// platform-wide because `dr_drive_storage_provider_kind` has no tenant column.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageOverviewResponse {
    pub(crate) generated_at: String,
    pub(crate) scope_tenant_id: String,
    pub(crate) capacity: StorageOverviewCapacityResponse,
    pub(crate) providers: StorageOverviewProvidersResponse,
    pub(crate) bindings: StorageOverviewBindingsResponse,
    pub(crate) catalog: StorageOverviewCatalogResponse,
    pub(crate) trend: Vec<StorageOverviewTrendPointResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageOverviewCapacityResponse {
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) total_object_count: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) active_object_count: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) deleted_object_count: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) used_bytes: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) average_object_bytes: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64::option")]
    pub(crate) largest_object_bytes: Option<i64>,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) bucket_count: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64::option")]
    pub(crate) quota_bytes: Option<i64>,
    pub(crate) quota_configured: bool,
    pub(crate) quota_usage_ratio: Option<f64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageOverviewProvidersResponse {
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) total_count: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) active_count: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) disabled_count: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) deleted_count: i64,
    pub(crate) usage: Vec<StorageOverviewProviderUsageResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageOverviewProviderUsageResponse {
    pub(crate) provider_id: String,
    pub(crate) name: String,
    pub(crate) provider_kind: String,
    pub(crate) status: String,
    pub(crate) bucket: String,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) object_count: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) used_bytes: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) binding_count: i64,
    pub(crate) is_tenant_default: bool,
    pub(crate) capacity_share: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageOverviewBindingsResponse {
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) total_count: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) active_count: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) inactive_count: i64,
    pub(crate) by_scope: StorageOverviewBindingScopeCountsResponse,
    pub(crate) has_tenant_default: bool,
    pub(crate) tenant_default_binding_id: Option<String>,
    pub(crate) tenant_default_provider_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageOverviewBindingScopeCountsResponse {
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) tenant_count: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) space_count: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) space_type_count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageOverviewCatalogResponse {
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) total_count: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) enabled_count: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) disabled_count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageOverviewTrendPointResponse {
    pub(crate) period_label: String,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) object_count: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) bytes: i64,
}

// ---------------------------------------------------------------------------
// Cross-provider storage migration
// ---------------------------------------------------------------------------

/// Open a migration run.
///
/// `id` is caller-supplied, matching `CreateStorageProviderRequest`: the caller
/// owns idempotency, so a retried plan cannot create a second run for the same
/// intended migration.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateStorageMigrationRequest {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) source_provider_id: String,
    pub(crate) target_provider_id: String,
    /// Optional target bucket; defaults to the target provider's own bucket.
    pub(crate) target_bucket: Option<String>,
    /// Re-point objects and bindings once every object has been copied.
    #[serde(default)]
    pub(crate) apply_binding_switch: bool,
}

/// Drive a run forward by one bounded batch.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RunStorageMigrationRequest {
    /// Objects to copy in this call. Clamped into a supported range rather than
    /// rejected: this is a throughput knob, not a contract.
    pub(crate) batch_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct StorageMigrationQuery {
    pub(crate) status: Option<String>,
    pub(crate) page_size: Option<i64>,
    pub(crate) page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct StorageMigrationItemQuery {
    pub(crate) status: Option<String>,
    pub(crate) page_size: Option<i64>,
    pub(crate) page_token: Option<String>,
}

/// Cancel takes no body.
///
/// The operator is *not* accepted from the client: it is a request-context
/// field, resolved from the verified `WebRequestContext`. Accepting a client
/// value would let a caller file an audit row under someone else's name, which
/// is exactly what the schema gate forbids.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CancelStorageMigrationRequest {}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageMigrationResponse {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) source_provider_id: String,
    pub(crate) target_provider_id: String,
    pub(crate) target_bucket: Option<String>,
    pub(crate) status: String,
    pub(crate) apply_binding_switch: bool,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) objects_total: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) objects_copied: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) objects_failed: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) objects_outstanding: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) bytes_copied: i64,
    pub(crate) progress_ratio: f64,
    pub(crate) failure_message: Option<String>,
    pub(crate) created_by: String,
    pub(crate) updated_by: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageMigrationItemResponse {
    pub(crate) id: String,
    pub(crate) storage_object_id: String,
    pub(crate) source_provider_id: String,
    pub(crate) source_bucket: String,
    pub(crate) source_object_key: String,
    pub(crate) target_provider_id: String,
    pub(crate) target_bucket: String,
    pub(crate) target_object_key: String,
    pub(crate) content_type: String,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) content_length: i64,
    pub(crate) checksum_sha256_hex: String,
    pub(crate) status: String,
    pub(crate) verified_checksum_sha256_hex: Option<String>,
    pub(crate) failure_message: Option<String>,
}

/// Report returned by a drive call, so the caller learns what this batch did
/// without a second round trip.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageMigrationRunResponse {
    pub(crate) migration: StorageMigrationResponse,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) copied_this_batch: i64,
    #[serde(with = "sdkwork_utils_rust::serde_int64")]
    pub(crate) failed_this_batch: i64,
    pub(crate) completed: bool,
}
