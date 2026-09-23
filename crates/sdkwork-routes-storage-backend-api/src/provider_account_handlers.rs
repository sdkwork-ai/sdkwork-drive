//! Admin storage routes over the platform service-provider account center.
//!
//! The account center itself is IAM-owned (`iam_provider_account` +
//! `iam_provider_credential`, domain logic in `sdkwork-iam-provider-account-service`).
//! The storage plane is one of its consumers: it reads accounts so operators
//! can bind a reusable account to a storage provider instead of embedding a
//! per-provider credential copy, and it can register a new account + access
//! key pair on the operator's behalf. Secret material is sealed by the account
//! service and never returned; these routes only ever project the masked view.
//!
//! Accounts live at one of three scopes and the storage console surfaces all
//! three, because which one is right depends on who owns the cloud bill:
//!
//! * `platform` — a global default an operator published once. Readable from
//!   every tenant, so it is what makes one Alibaba Cloud account reusable
//!   across the whole installation.
//! * `tenant` — the application tenant's own default, managed by its
//!   administrator and reused by every member.
//! * `user` — a personal account, so an end user can bring their own cloud
//!   credentials without asking the tenant administrator for anything.

use crate::app_context::DriveRequestContext;
use crate::dto::{
    CreateStorageProviderAccountRequest, ListStorageProviderAccountsQuery,
    StorageProviderAccountResponse,
};
use crate::error::{
    invalid_json_problem, map_provider_account_error, problem, ProblemDetail, SdkWorkResultCode,
};
use crate::provider_mappers::map_storage_provider_account;
use crate::response::{success_list_page_simple, StorageListHttpResponse};
use crate::state::AdminStorageState;
use crate::validators::{next_page_token, parse_offset_page};
use axum::extract::rejection::JsonRejection;
use axum::extract::{Extension, Query, State};
use axum::http::StatusCode;
use axum::Json;
use sdkwork_iam_provider_account_service::{
    create_account, list_accounts, resolve_account_scope, upsert_active_credential,
    AccountVisibility, NewProviderAccount, NewProviderCredential, ProviderAccountError,
    ScopeCaller, ACCOUNT_SCOPE_USER, ACCOUNT_TYPE_LONG_TERM_KEY, CAPABILITY_OBJECT_STORAGE,
    CREDENTIAL_KIND_ACCESS_KEY_PAIR, DEFAULT_CREDENTIAL_NAME, DEFAULT_ORGANIZATION_ID,
    PLATFORM_TENANT_ID,
};

pub(crate) async fn list_storage_provider_accounts(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Query(query): Query<ListStorageProviderAccountsQuery>,
) -> Result<
    StorageListHttpResponse<StorageProviderAccountResponse>,
    (StatusCode, Json<ProblemDetail>),
> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let operator_id = ctx.resolve_operator_id()?;
    let page = parse_offset_page(query.page_size, query.page_token)?;

    // `mine=true` is the console's "my accounts" view: it pins the listing to
    // the caller's personal scope so the client never has to know its own user
    // id. Asking for somebody else's user scope is a client error rather than
    // a silently empty page, because the two look identical to an operator.
    let mine = query.mine.unwrap_or(false);
    let requested_owner = query
        .owner_user_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if let Some(owner) = requested_owner {
        if owner != operator_id {
            return Err(map_provider_account_error(
                ProviderAccountError::Validation("ownerUserId must be the calling user".to_owned()),
            ));
        }
    }
    let (scope_type, owner_user_id) = if mine {
        (
            Some(ACCOUNT_SCOPE_USER.to_owned()),
            Some(operator_id.clone()),
        )
    } else {
        (
            query
                .scope_type
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned),
            requested_owner.map(str::to_owned),
        )
    };

    let visibility = AccountVisibility {
        tenant_id: tenant_id.clone(),
        user_id: Some(operator_id),
        // The storage console has no organization dimension: `DriveRequestContext`
        // carries a tenant and an actor only, so the organization layer is left off
        // rather than switched on with an id nobody supplied.
        organization_id: None,
        // Defaults to on: the global defaults are precisely what a tenant is
        // meant to reuse, so hiding them by default would defeat the point.
        include_platform: query.include_platform.unwrap_or(true),
        // The tenant-wide layer too, for the same reason: this console *is* the
        // tenant's account centre, and `drive.storage.admin` is the gate that
        // decided the caller may see it before the request got here.
        include_tenant_shared: true,
        include_organization_shared: false,
        scope_type,
        owner_user_id,
    };

    let (accounts, _total) = list_accounts(
        &state.pool,
        &visibility,
        None,
        query.vendor_code.as_deref(),
        query.status.as_deref(),
        query.search.as_deref(),
        page.limit,
        page.offset,
    )
    .await
    .map_err(map_provider_account_error)?;
    // The account center is business-agnostic; the storage console only binds
    // accounts that declare the object-storage capability, so the filter is
    // applied here instead of asking every consumer to post-filter.
    let capability = query.capability_code.as_deref().map(str::trim);
    let mut items = accounts
        .into_iter()
        .filter(|account| {
            capability.is_none_or(|wanted| {
                wanted.is_empty() || account.capability_codes.iter().any(|code| code == wanted)
            })
        })
        .map(map_storage_provider_account)
        .collect::<Vec<_>>();
    let next_page_token = next_page_token(&mut items, page);
    Ok(success_list_page_simple(items, page, next_page_token))
}

pub(crate) async fn create_storage_provider_account(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    payload: Result<Json<CreateStorageProviderAccountRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<StorageProviderAccountResponse>), (StatusCode, Json<ProblemDetail>)> {
    let Json(payload) = payload.map_err(invalid_json_problem)?;
    let operator_id = ctx.resolve_operator_id()?;
    let tenant_id = ctx.resolve_tenant_id()?;

    if payload.access_key_id.trim().is_empty() {
        return Err(map_provider_account_error(
            ProviderAccountError::Validation("accessKeyId is required".to_owned()),
        ));
    }
    if payload.secret_access_key.trim().is_empty() {
        return Err(map_provider_account_error(
            ProviderAccountError::Validation("secretAccessKey is required".to_owned()),
        ));
    }

    // A platform-scope account is resolvable from every tenant, so minting one
    // is a platform-operator action. Rejecting it here keeps the failure a
    // clear 403 instead of the 400 the domain guard would produce.
    let requested_scope = payload
        .scope_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if requested_scope.is_some_and(|scope| scope.eq_ignore_ascii_case("platform"))
        && tenant_id.trim() != PLATFORM_TENANT_ID
    {
        return Err(problem(
            StatusCode::FORBIDDEN,
            "provider account scope forbidden",
            "scopeType=platform is reserved for platform operators",
            SdkWorkResultCode::PermissionRequired,
        ));
    }
    // The domain owns the scope rules (platform tenant guard, user owner
    // defaulting, "owner must not be somebody else"). The storage console is an
    // admin surface — `can_invoke_drive_storage_operation` already required
    // `drive.storage.admin` before this handler ran — so it asserts the shared-scope
    // right rather than re-deriving it from a browser grant list it does not have.
    let caller = ScopeCaller::new(
        &tenant_id,
        Some(&operator_id),
        Some(DEFAULT_ORGANIZATION_ID),
    )
    .managing_shared(true);
    let resolved = resolve_account_scope(
        payload.scope_type.as_deref(),
        payload.owner_user_id.as_deref(),
        &caller,
        Some(DEFAULT_ORGANIZATION_ID),
    )
    .map_err(map_provider_account_error)?;
    let (scope_type, owner_user_id) = (resolved.scope_type, resolved.owner_user_id);

    let account = create_account(
        &state.pool,
        &NewProviderAccount {
            tenant_id: tenant_id.clone(),
            organization_id: resolved.organization_id,
            scope_type,
            owner_user_id,
            is_default: payload.is_default.unwrap_or(false),
            vendor_code: payload.vendor_code,
            account_code: payload.account_code,
            display_name: payload.display_name,
            // This console only ever writes an access-key pair, and an access-key
            // pair is a long-term key, so the identity shape is *derived* rather than
            // client-chosen. Letting the caller name it would let the console record a
            // shape the credential does not have; a disagreement is therefore a client
            // error rather than something to quietly overwrite.
            account_type: {
                if let Some(requested) = payload.account_type.as_deref() {
                    let requested = requested.trim();
                    if !requested.is_empty() && requested != ACCOUNT_TYPE_LONG_TERM_KEY {
                        return Err(map_provider_account_error(
                            ProviderAccountError::Validation(format!(
                                "accountType `{requested}` does not match the access-key pair this \
                                 console stores; a long-term key is `{ACCOUNT_TYPE_LONG_TERM_KEY}`"
                            )),
                        ));
                    }
                }
                ACCOUNT_TYPE_LONG_TERM_KEY.to_owned()
            },
            environment: payload
                .environment
                .unwrap_or_else(|| "production".to_owned()),
            external_account_id: payload.external_account_id,
            // Accounts minted from the storage console are created for the
            // storage capability; the column stays free-form so a later
            // capability can be added without a schema change.
            capability_codes: vec![CAPABILITY_OBJECT_STORAGE.to_owned()],
            region_code: payload.region_code,
            actor_id: operator_id.clone(),
        },
        &caller,
    )
    .await
    .map_err(map_provider_account_error)?;

    // The access key pair is sealed straight into the write-only credential
    // row; this route never echoes it back.
    upsert_active_credential(
        &state.pool,
        &NewProviderCredential {
            provider_account_id: account.id.clone(),
            credential_kind: CREDENTIAL_KIND_ACCESS_KEY_PAIR.to_owned(),
            credential_name: DEFAULT_CREDENTIAL_NAME.to_owned(),
            access_key_id: Some(payload.access_key_id),
            secret_access_key: Some(payload.secret_access_key),
            session_token: payload.session_token,
            secret_text: None,
            expires_at: None,
            actor_id: operator_id,
        },
    )
    .await
    .map_err(map_provider_account_error)?;

    // Re-read so credential_configured / credential_count reflect the newly
    // written credential row. The account's own tenant is used rather than the
    // caller's: a platform-scope account lives in the platform tenant.
    let account = sdkwork_iam_provider_account_service::find_account(
        &state.pool,
        &account.tenant_id,
        &account.id,
    )
    .await
    .map_err(map_provider_account_error)?
    .ok_or_else(|| {
        map_provider_account_error(ProviderAccountError::Unavailable(
            "created provider account not readable".to_owned(),
        ))
    })?;

    Ok((
        StatusCode::CREATED,
        Json(map_storage_provider_account(account)),
    ))
}
