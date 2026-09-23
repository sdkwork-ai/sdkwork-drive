//! Built-in storage provider bootstrap.
//!
//! The storage console starts on an empty plane: the provider-kind catalog is
//! initializable (`storageProviderKinds.create`) but nothing behind a kind
//! exists, so every "test connection" button has nothing to test and an
//! operator has to hand-write a provider configuration *and* an account-center
//! account before the first byte can be moved. This route is that missing half.
//!
//! One run, per built-in **cloud** kind:
//!
//! 1. ensure an account-center account for the kind's vendor, carrying a
//!    **placeholder access-key pair** in the vendor's own key shape;
//! 2. ensure a provider configuration bound to that account (`provider_account_id`),
//!    so the credential source is the reusable account center rather than a
//!    per-provider copy;
//!
//! and once for the tenant: ensure the `local_filesystem` provider and make it
//! the tenant's default binding, so a fresh environment can store objects
//! without any cloud credential at all.
//!
//! Two rules make this safe to run repeatedly and safe to run on a live
//! installation:
//!
//! * **Nothing existing is overwritten.** A run creates what is missing and
//!   leaves everything else untouched — in particular it never rewrites the
//!   credential of an account that already exists, because that account may
//!   hold the operator's real keys by then. `credentialSeeded=false` in the
//!   response says exactly that: "this account kept its own credential".
//! * **Placeholder accounts are not made the vendor default.** `is_default`
//!   resolution clears the previous default in the same scope, so publishing a
//!   placeholder as default would silently re-point every consumer of that
//!   vendor at a stub credential. Providers reference their account by id
//!   instead, which is a pinned lookup and needs no default.
//!
//! ## Why the account is written here rather than seeded
//!
//! Credential material is sealed with a fail-closed AES-256-GCM envelope keyed
//! by `SDKWORK_IAM_PROVIDER_CREDENTIAL_MASTER_SECRET`, which a SQL seed cannot
//! produce. Seeding the accounts therefore has to be a runtime write through
//! `sdkwork-iam-provider-account-service` — the same call the console's own
//! "new account" form makes.

use crate::app_context::DriveRequestContext;
use crate::audit::record_audit_event;
use crate::dto::{OffsetPage, StorageProviderAccountDefaultResponse};
use crate::error::{map_provider_account_error, map_service_error, ProblemDetail};
use crate::provider_mappers::parse_storage_provider_kind;
use crate::response::{success_list_page_simple, StorageListHttpResponse};
use crate::state::AdminStorageState;
use crate::validators::{
    default_storage_provider_binding_id, default_storage_root_prefix,
    storage_provider_binding_purpose, storage_provider_binding_scope, StorageProviderBindingTarget,
};
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::Json;
use sdkwork_drive_contract::drive::domain_events::admin_audit;
use sdkwork_drive_workspace_service::application::storage_provider_kind_service::DriveStorageProviderKindService;
use sdkwork_drive_workspace_service::application::storage_provider_service::{
    CreateStorageProviderCommand, DriveStorageProviderService, GetStorageProviderCommand,
};
use sdkwork_drive_workspace_service::infrastructure::sql::storage_provider_kind_store::SqlStorageProviderKindStore;
use sdkwork_drive_workspace_service::infrastructure::sql::storage_provider_store::SqlStorageProviderStore;
use sdkwork_drive_workspace_service::DriveServiceError;
use sdkwork_iam_provider_account_service::{
    create_account, list_accounts, resolve_account_scope, upsert_active_credential,
    AccountVisibility, NewProviderAccount, NewProviderCredential, ScopeCaller,
    ACCOUNT_SCOPE_PLATFORM, ACCOUNT_SCOPE_TENANT, ACCOUNT_TYPE_LONG_TERM_KEY,
    CAPABILITY_OBJECT_STORAGE, CREDENTIAL_KIND_ACCESS_KEY_PAIR, DEFAULT_CREDENTIAL_NAME,
    DEFAULT_ORGANIZATION_ID, PLATFORM_TENANT_ID,
};

/// Provider id prefix every bootstrapped row shares, so an operator can tell a
/// built-in row from one they created by hand. Also what makes the run
/// idempotent: the id is derived from the kind, never random.
const BUILTIN_PROVIDER_ID_PREFIX: &str = "builtin-storage-provider-";

const LOCAL_PROVIDER_ID: &str = "builtin-storage-provider-local-filesystem";
const LOCAL_PROVIDER_KIND: &str = "local_filesystem";
const LOCAL_PROVIDER_NAME: &str = "Built-in Local Filesystem";
/// `local_filesystem` stores under a filesystem path, not a bucket URL, and the
/// DDL requires `file://` plus `strict_tls = FALSE`.
const LOCAL_PROVIDER_ENDPOINT: &str = "file:///var/lib/sdkwork-drive";
/// `dr_drive_storage_provider.bucket` is format-checked but not existence-checked,
/// so a placeholder only has to be a legal name; the operator renames it when
/// pointing the row at a real bucket.
const LOCAL_PROVIDER_BUCKET: &str = "sdkwork-drive";

/// A built-in cloud provider kind, with the account it is bound to.
///
/// The endpoint and region mirror the console's kind picker
/// (`providerKindConfig.ts`), but they are *not* the same thing: that table is
/// presentation (it auto-fills a form the operator then edits), while these are
/// the values written into a row. Keeping the whole bootstrap — kind, vendor,
/// endpoint, key shape — in one table here is what keeps it readable.
struct BuiltinCloudProvider {
    /// `dr_drive_storage_provider.provider_kind`, and the suffix of the row id.
    provider_kind: &'static str,
    provider_name: &'static str,
    endpoint_url: &'static str,
    region: &'static str,
    bucket: &'static str,
    /// `iam_provider_account.vendor_code`.
    vendor_code: &'static str,
    /// Stable account code: what makes a re-run recognise its own account
    /// instead of creating a second one.
    account_code: &'static str,
    account_display_name: &'static str,
    /// Placeholder key material, in the vendor's own identifier shape so an
    /// operator can see at a glance that it is a placeholder.
    access_key_id: &'static str,
    secret_access_key: &'static str,
}

const BUILTIN_CLOUD_PROVIDERS: [BuiltinCloudProvider; 6] = [
    BuiltinCloudProvider {
        provider_kind: "s3_compatible",
        provider_name: "Built-in Amazon S3",
        endpoint_url: "https://s3.us-east-1.amazonaws.com",
        region: "us-east-1",
        bucket: "sdkwork-drive-s3-compatible",
        vendor_code: "aws",
        account_code: "builtin-aws-storage",
        account_display_name: "Built-in AWS storage account",
        // AWS's own documentation sample pair: unmistakably a placeholder, and
        // shaped exactly like a real access key id / secret.
        access_key_id: "AKIAIOSFODNN7EXAMPLE",
        secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
    },
    BuiltinCloudProvider {
        provider_kind: "aliyun_oss",
        provider_name: "Built-in Alibaba Cloud OSS",
        endpoint_url: "https://oss-cn-hangzhou.aliyuncs.com",
        region: "cn-hangzhou",
        bucket: "sdkwork-drive-aliyun-oss",
        vendor_code: "aliyun",
        account_code: "builtin-aliyun-storage",
        account_display_name: "Built-in Alibaba Cloud storage account",
        access_key_id: "LTAI5tPLACEHOLDER0000",
        secret_access_key: "PLACEHOLDER0000000000000000000000",
    },
    BuiltinCloudProvider {
        provider_kind: "tencent_cos",
        provider_name: "Built-in Tencent Cloud COS",
        endpoint_url: "https://cos.ap-guangzhou.myqcloud.com",
        region: "ap-guangzhou",
        bucket: "sdkwork-drive-cos-1250000000",
        vendor_code: "tencent",
        account_code: "builtin-tencent-storage",
        account_display_name: "Built-in Tencent Cloud storage account",
        access_key_id: "AKIDPLACEHOLDER00000000",
        secret_access_key: "PLACEHOLDER0000000000000000000000",
    },
    BuiltinCloudProvider {
        provider_kind: "huawei_obs",
        provider_name: "Built-in Huawei Cloud OBS",
        endpoint_url: "https://obs.cn-north-1.myhuaweicloud.com",
        region: "cn-north-1",
        bucket: "sdkwork-drive-huawei-obs",
        vendor_code: "huawei",
        account_code: "builtin-huawei-storage",
        account_display_name: "Built-in Huawei Cloud storage account",
        access_key_id: "PLACEHOLDER00000000",
        secret_access_key: "PLACEHOLDER0000000000000000000000000000000000",
    },
    BuiltinCloudProvider {
        provider_kind: "volcengine_tos",
        provider_name: "Built-in Volcengine TOS",
        endpoint_url: "https://tos-cn-beijing.volces.com",
        region: "cn-beijing",
        bucket: "sdkwork-drive-volcengine-tos",
        vendor_code: "volcengine",
        account_code: "builtin-volcengine-storage",
        account_display_name: "Built-in Volcengine storage account",
        access_key_id: "AKPLACEHOLDER00000000",
        secret_access_key: "PLACEHOLDER0000000000000000000000",
    },
    BuiltinCloudProvider {
        provider_kind: "google_cloud_storage",
        provider_name: "Built-in Google Cloud Storage",
        endpoint_url: "https://storage.googleapis.com",
        region: "us-central1",
        bucket: "sdkwork-drive-gcs",
        vendor_code: "google",
        account_code: "builtin-google-storage",
        account_display_name: "Built-in Google Cloud storage account",
        access_key_id: "GOOGPLACEHOLDER0000000000000000000000000000000000000000000000",
        secret_access_key: "PLACEHOLDER0000000000000000000000000000000000000000",
    },
];

/// Offset page covering every built-in kind in one response: seven rows at most,
/// so the endpoint is not paginated — it answers "what does the plane look like
/// now", not "give me a slice of a large set".
const ALL_BUILTIN_KINDS_PAGE: OffsetPage = OffsetPage {
    limit: 100,
    offset: 0,
};

fn provider_id_for(provider_kind: &str) -> String {
    format!(
        "{BUILTIN_PROVIDER_ID_PREFIX}{}",
        provider_kind.replace('_', "-")
    )
}

fn provider_service(
    state: &AdminStorageState,
) -> DriveStorageProviderService<SqlStorageProviderStore> {
    DriveStorageProviderService::new(SqlStorageProviderStore::new(state.pool.clone()))
}

/// Whether a provider row already exists, without treating "absent" as an error.
async fn provider_exists(
    state: &AdminStorageState,
    provider_id: &str,
) -> Result<bool, (StatusCode, Json<ProblemDetail>)> {
    match provider_service(state)
        .get_storage_provider(GetStorageProviderCommand {
            provider_id: provider_id.to_owned(),
        })
        .await
    {
        Ok(_) => Ok(true),
        Err(DriveServiceError::NotFound(_)) => Ok(false),
        Err(error) => Err(map_service_error(error)),
    }
}

/// Find this run's account for one vendor, if it is already there.
///
/// The listing is pinned to the writing scope rather than searched across it: a
/// placeholder account some other tenant owns is not this run's to reuse, and
/// matching on the account code alone would bind this tenant's provider to a
/// foreign account.
async fn find_existing_account_id(
    state: &AdminStorageState,
    operator_id: &str,
    account_tenant_id: &str,
    scope_type: &str,
    account_code: &str,
) -> Result<Option<String>, (StatusCode, Json<ProblemDetail>)> {
    let visibility = AccountVisibility {
        tenant_id: account_tenant_id.to_owned(),
        user_id: Some(operator_id.to_owned()),
        organization_id: None,
        // A platform-scope account lives in the platform tenant and is only
        // reachable through this flag; a tenant-scope one is reached directly.
        include_platform: scope_type == ACCOUNT_SCOPE_PLATFORM,
        include_tenant_shared: true,
        include_organization_shared: false,
        scope_type: Some(scope_type.to_owned()),
        owner_user_id: None,
    };
    let (accounts, _total) = list_accounts(
        &state.pool,
        &visibility,
        None,
        None,
        None,
        Some(account_code),
        50,
        0,
    )
    .await
    .map_err(map_provider_account_error)?;
    Ok(accounts
        .into_iter()
        .find(|account| account.account_code == account_code)
        .map(|account| account.id))
}

/// Ensure the account-center account for one cloud provider kind.
///
/// Returns `(account_id, account_created, credential_seeded)`.
async fn ensure_vendor_account(
    state: &AdminStorageState,
    tenant_id: &str,
    operator_id: &str,
    builtin: &BuiltinCloudProvider,
) -> Result<(String, bool, bool), (StatusCode, Json<ProblemDetail>)> {
    // A platform-scope account is resolvable from every tenant, which is what
    // makes one placeholder account reusable across the whole installation; it
    // may only be minted from inside the platform tenant, so a tenant console
    // gets the tenant-scope form instead of a refusal.
    let scope_type = if tenant_id.trim() == PLATFORM_TENANT_ID {
        ACCOUNT_SCOPE_PLATFORM
    } else {
        ACCOUNT_SCOPE_TENANT
    };
    // `create_account` moves a platform-scope row into the platform tenant, so
    // the lookup has to look there too or a re-run would create a duplicate.
    let account_tenant_id = if scope_type == ACCOUNT_SCOPE_PLATFORM {
        PLATFORM_TENANT_ID
    } else {
        tenant_id
    };

    if let Some(existing) = find_existing_account_id(
        state,
        operator_id,
        account_tenant_id,
        scope_type,
        builtin.account_code,
    )
    .await?
    {
        // Existing account: keep it, and in particular keep its credential. By
        // the time a bootstrap is re-run the operator may already have replaced
        // the placeholder with real keys, and rewriting them would break the very
        // provider this run is meant to help.
        return Ok((existing, false, false));
    }

    let caller = ScopeCaller::new(tenant_id, Some(operator_id), Some(DEFAULT_ORGANIZATION_ID))
        .managing_shared(true);
    let resolved = resolve_account_scope(
        Some(scope_type),
        None,
        &caller,
        Some(DEFAULT_ORGANIZATION_ID),
    )
    .map_err(map_provider_account_error)?;

    let account = create_account(
        &state.pool,
        &NewProviderAccount {
            tenant_id: tenant_id.to_owned(),
            organization_id: resolved.organization_id,
            scope_type: resolved.scope_type,
            owner_user_id: resolved.owner_user_id,
            // See the module docs: publishing a placeholder as the vendor
            // default would re-point every consumer of that vendor at a stub.
            is_default: false,
            vendor_code: builtin.vendor_code.to_owned(),
            account_code: builtin.account_code.to_owned(),
            display_name: builtin.account_display_name.to_owned(),
            account_type: ACCOUNT_TYPE_LONG_TERM_KEY.to_owned(),
            environment: "production".to_owned(),
            external_account_id: None,
            capability_codes: vec![CAPABILITY_OBJECT_STORAGE.to_owned()],
            region_code: Some(builtin.region.to_owned()),
            actor_id: operator_id.to_owned(),
        },
        &caller,
    )
    .await
    .map_err(map_provider_account_error)?;

    upsert_active_credential(
        &state.pool,
        &NewProviderCredential {
            provider_account_id: account.id.clone(),
            credential_kind: CREDENTIAL_KIND_ACCESS_KEY_PAIR.to_owned(),
            credential_name: DEFAULT_CREDENTIAL_NAME.to_owned(),
            access_key_id: Some(builtin.access_key_id.to_owned()),
            secret_access_key: Some(builtin.secret_access_key.to_owned()),
            session_token: None,
            secret_text: None,
            expires_at: None,
            actor_id: operator_id.to_owned(),
        },
    )
    .await
    .map_err(map_provider_account_error)?;

    Ok((account.id, true, true))
}

/// Ensure one provider configuration exists, creating it when absent.
#[allow(clippy::too_many_arguments)]
async fn ensure_provider(
    state: &AdminStorageState,
    tenant_id: &str,
    operator_id: &str,
    provider_id: &str,
    provider_kind: &str,
    name: &str,
    endpoint_url: &str,
    region: Option<&str>,
    bucket: &str,
    provider_account_id: Option<&str>,
) -> Result<bool, (StatusCode, Json<ProblemDetail>)> {
    if provider_exists(state, provider_id).await? {
        return Ok(false);
    }
    let kind = parse_storage_provider_kind(provider_kind).map_err(map_service_error)?;
    provider_service(state)
        .create_storage_provider(CreateStorageProviderCommand {
            id: provider_id.to_owned(),
            tenant_id: tenant_id.to_owned(),
            provider_kind: kind,
            name: name.to_owned(),
            endpoint_url: endpoint_url.to_owned(),
            region: region.map(str::to_owned),
            bucket: bucket.to_owned(),
            // `None` lets the workspace service apply its per-kind defaults,
            // which is the same rule the console form uses.
            path_style: None,
            strict_tls: None,
            credential_ref: None,
            provider_account_id: provider_account_id.map(str::to_owned),
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: Some("active".to_owned()),
            operator_id: operator_id.to_owned(),
        })
        .await
        .map_err(map_service_error)?;
    Ok(true)
}

/// Ensure the tenant default binding exists, without disturbing an existing one.
///
/// Returns whether this run wrote the binding.
async fn ensure_default_binding(
    state: &AdminStorageState,
    tenant_id: &str,
    operator_id: &str,
    provider_id: &str,
) -> Result<bool, (StatusCode, Json<ProblemDetail>)> {
    let target = StorageProviderBindingTarget::Tenant;
    let binding_id = default_storage_provider_binding_id(tenant_id, &target);
    let purpose = storage_provider_binding_purpose(&target);
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM dr_drive_storage_provider_binding
         WHERE tenant_id = $1 AND id = $2 AND purpose = $3 AND lifecycle_status != 'deleted'",
    )
    .bind(tenant_id)
    .bind(&binding_id)
    .bind(&purpose)
    .fetch_one(&state.pool)
    .await
    .map_err(|error| {
        map_service_error(DriveServiceError::Internal(format!(
            "read dr_drive_storage_provider_binding failed: {error}"
        )))
    })?;
    if existing > 0 {
        // An operator-chosen default is a decision, not a gap: leave it.
        return Ok(false);
    }

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider_binding (
            id, tenant_id, space_id, provider_id, binding_scope, purpose,
            storage_root_prefix, lifecycle_status, version, created_by, updated_by
         ) VALUES ($1, $2, NULL, $3, $4, $5, $6, 'active', 1, $7, $7)
         ON CONFLICT(id) DO NOTHING",
    )
    .bind(&binding_id)
    .bind(tenant_id)
    .bind(provider_id)
    .bind(storage_provider_binding_scope(&target))
    .bind(&purpose)
    // Reuse the same derivation the binding routes use. Duplicating the literal
    // here would let this default and the operator-facing one drift apart the
    // day the object layout changes.
    .bind(default_storage_root_prefix(tenant_id, &target))
    .bind(operator_id)
    .execute(&state.pool)
    .await
    .map_err(|error| {
        map_service_error(DriveServiceError::Internal(format!(
            "insert dr_drive_storage_provider_binding failed: {error}"
        )))
    })?;
    Ok(true)
}

/// Bootstrap the built-in storage plane: an account per cloud provider kind, its
/// provider configuration, and a usable tenant default binding.
pub(crate) async fn initialize_storage_provider_account_defaults(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
) -> Result<
    StorageListHttpResponse<StorageProviderAccountDefaultResponse>,
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;

    let kinds =
        DriveStorageProviderKindService::new(SqlStorageProviderKindStore::new(state.pool.clone()))
            .list_storage_provider_kinds()
            .await
            .map_err(map_service_error)?;
    // A kind the operator disabled is a decision: it must not come back to life
    // through a bootstrap run, so it is skipped and simply absent from the
    // response.
    let kind_is_enabled = |provider_kind: &str| {
        kinds
            .iter()
            .any(|summary| summary.kind.provider_kind == provider_kind && summary.kind.enabled)
    };

    let mut items = Vec::with_capacity(BUILTIN_CLOUD_PROVIDERS.len() + 1);

    // The credential-free kind is settled first: it is what the default binding
    // points at, so it has to exist before the binding is written.
    let local_provider_created = if kind_is_enabled(LOCAL_PROVIDER_KIND) {
        ensure_provider(
            &state,
            &tenant_id,
            &operator_id,
            LOCAL_PROVIDER_ID,
            LOCAL_PROVIDER_KIND,
            LOCAL_PROVIDER_NAME,
            LOCAL_PROVIDER_ENDPOINT,
            None,
            LOCAL_PROVIDER_BUCKET,
            None,
        )
        .await?
    } else {
        false
    };
    let local_provider_available = if local_provider_created {
        true
    } else {
        kind_is_enabled(LOCAL_PROVIDER_KIND) && provider_exists(&state, LOCAL_PROVIDER_ID).await?
    };

    for builtin in &BUILTIN_CLOUD_PROVIDERS {
        if !kind_is_enabled(builtin.provider_kind) {
            continue;
        }
        let provider_id = provider_id_for(builtin.provider_kind);
        let (account_id, account_created, credential_seeded) =
            ensure_vendor_account(&state, &tenant_id, &operator_id, builtin).await?;
        let provider_created = ensure_provider(
            &state,
            &tenant_id,
            &operator_id,
            &provider_id,
            builtin.provider_kind,
            builtin.provider_name,
            builtin.endpoint_url,
            Some(builtin.region),
            builtin.bucket,
            Some(&account_id),
        )
        .await?;
        items.push(StorageProviderAccountDefaultResponse {
            provider_kind: builtin.provider_kind.to_owned(),
            provider_id,
            provider_created,
            vendor_code: Some(builtin.vendor_code.to_owned()),
            provider_account_id: Some(account_id),
            account_code: Some(builtin.account_code.to_owned()),
            account_created,
            credential_seeded,
        });
    }

    // Only when the credential-free provider exists to point at; and
    // `ensure_default_binding` itself declines to touch an operator's default.
    if local_provider_available {
        ensure_default_binding(&state, &tenant_id, &operator_id, LOCAL_PROVIDER_ID).await?;
    }

    items.push(StorageProviderAccountDefaultResponse {
        provider_kind: LOCAL_PROVIDER_KIND.to_owned(),
        provider_id: LOCAL_PROVIDER_ID.to_owned(),
        provider_created: local_provider_created,
        vendor_code: None,
        provider_account_id: None,
        account_code: None,
        account_created: false,
        credential_seeded: false,
    });

    record_audit_event(
        &state,
        admin_audit::storage_provider_account::INITIALIZED,
        "storage_provider_account",
        &tenant_id,
        &operator_id,
    )
    .await?;

    Ok(success_list_page_simple(
        items,
        ALL_BUILTIN_KINDS_PAGE,
        None,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The placeholder must be recognisable *as* a placeholder by the operator who
    /// opens the account in the console. A value that merely happens to be fake is
    /// not enough: the whole point is that a real key can be told apart at a
    /// glance, so a real key pasted into this table would fail the suite.
    fn is_recognisable_placeholder(value: &str) -> bool {
        let upper = value.to_ascii_uppercase();
        upper.contains("PLACEHOLDER") || upper.contains("EXAMPLE")
    }

    /// Mirrors `ck_iam_provider_account_vendor_code` in the IAM baseline DDL.
    fn matches_vendor_code_contract(value: &str) -> bool {
        let mut chars = value.chars();
        match chars.next() {
            Some(first) if first.is_ascii_lowercase() => {}
            _ => return false,
        }
        let rest = chars.collect::<Vec<_>>();
        (2..=32).contains(&(rest.len() + 1))
            && rest
                .iter()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_')
    }

    /// Mirrors `ck_iam_provider_account_account_code` in the IAM baseline DDL.
    fn matches_account_code_contract(value: &str) -> bool {
        let mut chars = value.chars();
        match chars.next() {
            Some(first) if first.is_ascii_lowercase() || first.is_ascii_digit() => {}
            _ => return false,
        }
        let rest = chars.collect::<Vec<_>>();
        (2..=64).contains(&(rest.len() + 1))
            && rest.iter().all(|c| {
                c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_' || *c == '.' || *c == '-'
            })
    }

    /// Mirrors the non-filesystem branch of `ck_dr_drive_storage_provider_bucket`.
    ///
    /// The reserved-name branches (`xn--`, `sthree-`, `-s3alias`, …) are left out
    /// on purpose: no built-in row is anywhere near one, and modelling them here
    /// would only restate the DDL.
    fn matches_bucket_contract(value: &str) -> bool {
        let len = value.len();
        if !(3..=63).contains(&len) {
            return false;
        }
        let is_lower_alnum = |byte: u8| byte.is_ascii_lowercase() || byte.is_ascii_digit();
        let bytes = value.as_bytes();
        if !is_lower_alnum(bytes[0]) || !is_lower_alnum(bytes[len - 1]) {
            return false;
        }
        if !value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-')
        {
            return false;
        }
        !value.contains("..") && !value.contains(".-") && !value.contains("-.")
    }

    #[test]
    fn every_builtin_cloud_provider_names_a_real_domain_kind_exactly_once() {
        let kinds = BUILTIN_CLOUD_PROVIDERS
            .iter()
            .map(|builtin| builtin.provider_kind)
            .collect::<Vec<_>>();

        let mut unique = kinds.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(
            unique.len(),
            kinds.len(),
            "two built-in rows claim the same provider kind, so one would shadow the other: {kinds:?}"
        );

        for kind in &kinds {
            assert!(
                parse_storage_provider_kind(kind).is_ok(),
                "{kind} is not a domain provider kind; a bootstrap run would fail on it"
            );
        }

        // `local_filesystem` is settled separately (it has no account), so it must
        // not also appear here.
        assert!(
            !kinds.contains(&LOCAL_PROVIDER_KIND),
            "the credential-free kind must be handled by the local branch only"
        );
    }

    #[test]
    fn account_codes_are_stable_and_distinct_so_a_rerun_recognises_its_own_rows() {
        let codes = BUILTIN_CLOUD_PROVIDERS
            .iter()
            .map(|builtin| builtin.account_code)
            .collect::<Vec<_>>();

        let mut unique = codes.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(
            unique.len(),
            codes.len(),
            "duplicate account codes make two kinds share one account: {codes:?}"
        );

        for builtin in &BUILTIN_CLOUD_PROVIDERS {
            assert!(
                matches_vendor_code_contract(builtin.vendor_code),
                "{} has vendor_code {:?}, which the IAM CHECK would reject",
                builtin.provider_kind,
                builtin.vendor_code
            );
            assert!(
                matches_account_code_contract(builtin.account_code),
                "{} has account_code {:?}, which the IAM CHECK would reject",
                builtin.provider_kind,
                builtin.account_code
            );
            assert_eq!(
                builtin.account_display_name.trim(),
                builtin.account_display_name,
                "{} has an untrimmed display name, which the IAM CHECK would reject",
                builtin.provider_kind
            );
            assert!(
                (1..=128).contains(&builtin.account_display_name.chars().count()),
                "{} has a display name outside the IAM length bound",
                builtin.provider_kind
            );
        }
    }

    #[test]
    fn provider_ids_are_deterministic_prefixed_and_distinct() {
        for builtin in &BUILTIN_CLOUD_PROVIDERS {
            let id = provider_id_for(builtin.provider_kind);
            assert_eq!(
                id,
                provider_id_for(builtin.provider_kind),
                "an unstable provider id would create a second provider on every run"
            );
            assert!(
                id.starts_with(BUILTIN_PROVIDER_ID_PREFIX),
                "{id} is not namespaced, so an operator cannot tell it from a hand-made row"
            );
            assert_ne!(id, LOCAL_PROVIDER_ID);
            assert!(
                !id.contains('_'),
                "{id} leaks an underscore; ids are kebab-cased from the kind"
            );
        }

        // The id is the idempotency key, so the mapping has to be injective.
        let ids = BUILTIN_CLOUD_PROVIDERS
            .iter()
            .map(|builtin| provider_id_for(builtin.provider_kind))
            .collect::<Vec<_>>();
        let mut unique = ids.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), ids.len(), "provider id collision: {ids:?}");

        assert_eq!(
            provider_id_for("tencent_cos"),
            "builtin-storage-provider-tencent-cos"
        );
    }

    #[test]
    fn placeholder_credentials_keep_each_vendors_key_shape_and_are_not_identical() {
        for builtin in &BUILTIN_CLOUD_PROVIDERS {
            assert!(
                is_recognisable_placeholder(builtin.access_key_id),
                "{} ships access key id {:?}, which a reader could mistake for a real key",
                builtin.provider_kind,
                builtin.access_key_id
            );
            assert!(
                is_recognisable_placeholder(builtin.secret_access_key),
                "{} ships a secret access key that is not marked as a placeholder",
                builtin.provider_kind
            );
            assert!(
                !builtin.access_key_id.is_empty()
                    && !builtin.secret_access_key.is_empty()
                    && builtin.access_key_id != builtin.secret_access_key,
                "{} needs a pair of distinct, non-empty placeholder values",
                builtin.provider_kind
            );

            // The vendor shape is the requirement: an operator should see
            // "LTAI…" next to OSS and "AKIA…" next to S3 without reading docs.
            let expected_prefix = match builtin.vendor_code {
                "aws" => Some("AKIA"),
                "aliyun" => Some("LTAI"),
                "tencent" => Some("AKID"),
                "google" => Some("GOOG"),
                // Huawei OBS and Volcengine AKs have no distinctive prefix; the
                // placeholder marker above is the whole signal there.
                "huawei" | "volcengine" => None,
                other => panic!("{other} has no documented placeholder shape"),
            };
            if let Some(prefix) = expected_prefix {
                assert!(
                    builtin.access_key_id.starts_with(prefix),
                    "{} must use the {prefix} key shape, got {:?}",
                    builtin.provider_kind,
                    builtin.access_key_id
                );
            }
        }
    }

    #[test]
    fn bootstrap_provider_rows_satisfy_the_storage_ddl_contracts() {
        for builtin in &BUILTIN_CLOUD_PROVIDERS {
            assert!(
                builtin.endpoint_url.starts_with("https://"),
                "{} must bootstrap a TLS endpoint",
                builtin.provider_kind
            );
            assert!(
                matches_bucket_contract(builtin.bucket),
                "{} has bucket {:?}, which the storage CHECK would reject",
                builtin.provider_kind,
                builtin.bucket
            );
            assert!(!builtin.region.is_empty());
            assert_eq!(builtin.provider_name.trim(), builtin.provider_name);
        }

        // The credential-free default carries the `file://` endpoint the DDL
        // demands of a filesystem provider (`strict_tls = FALSE OR endpoint_url
        // ~* '^(https|file)://'` is satisfied either way by a `file://` URL).
        assert!(LOCAL_PROVIDER_ENDPOINT.starts_with("file://"));
        assert!(
            !LOCAL_PROVIDER_ENDPOINT.contains(char::is_whitespace),
            "the storage DDL rejects whitespace in endpoint_url"
        );
        assert!(
            matches_bucket_contract(LOCAL_PROVIDER_BUCKET),
            "the local provider's bucket is a legal non-filesystem bucket name too"
        );
        assert_eq!(LOCAL_PROVIDER_NAME.trim(), LOCAL_PROVIDER_NAME);

        // The local provider follows the same id derivation as every other
        // built-in row, so the whole built-in set is recognisable by one prefix.
        assert_eq!(LOCAL_PROVIDER_ID, provider_id_for(LOCAL_PROVIDER_KIND));
        assert!(LOCAL_PROVIDER_ID.starts_with(BUILTIN_PROVIDER_ID_PREFIX));
    }
}
