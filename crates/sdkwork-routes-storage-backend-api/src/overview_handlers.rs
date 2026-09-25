use crate::app_context::DriveRequestContext;
use crate::dto::{
    StorageOverviewBindingScopeCountsResponse, StorageOverviewBindingsResponse,
    StorageOverviewCapacityResponse, StorageOverviewCatalogResponse, StorageOverviewProvidersResponse,
    StorageOverviewProviderUsageResponse, StorageOverviewQuery, StorageOverviewResponse,
    StorageOverviewTrendPointResponse,
};
use crate::error::{map_service_error, validation_problem, ProblemDetail};
use crate::state::AdminStorageState;
use crate::validators::{default_storage_provider_binding_id, StorageProviderBindingTarget};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use sdkwork_drive_workspace_service::DriveServiceError;
use sqlx::Row;

/// Inclusive bounds for the trend window.
///
/// Anything outside the documented 1..24 range is a caller mistake rather than
/// a meaningful request, so it is rejected instead of being silently clamped to
/// a value the caller did not ask for.
const TREND_MONTHS_MIN: i64 = 1;
const TREND_MONTHS_MAX: i64 = 24;
const TREND_MONTHS_DEFAULT: i64 = 12;

pub(crate) async fn get_storage_overview(
    State(state): State<AdminStorageState>,
    Extension(ctx): Extension<DriveRequestContext>,
    Query(query): Query<StorageOverviewQuery>,
) -> Result<Json<StorageOverviewResponse>, (StatusCode, Json<ProblemDetail>)> {
    let tenant_id = ctx.resolve_tenant_id()?;
    let trend_months = resolve_trend_months(query.trend_months)?;

    // Resolved once: both the binding block and every provider row need it, and
    // two independent lookups would be free to disagree on an edge case.
    let tenant_default = load_tenant_default_binding(&state, &tenant_id).await?;

    let capacity = load_capacity(&state, &tenant_id).await?;
    let provider_counts = load_provider_counts(&state, &tenant_id).await?;
    let provider_usage =
        load_provider_usage(&state, &tenant_id, capacity.metrics.used_bytes, &tenant_default)
            .await?;
    let bindings = load_bindings(&state, &tenant_id, &tenant_default).await?;
    let catalog = load_catalog(&state).await?;
    let trend = load_trend(&state, &tenant_id, trend_months).await?;

    Ok(Json(StorageOverviewResponse {
        generated_at: capacity.generated_at,
        scope_tenant_id: tenant_id,
        capacity: capacity.metrics,
        providers: StorageOverviewProvidersResponse {
            total_count: provider_counts[0],
            active_count: provider_counts[1],
            disabled_count: provider_counts[2],
            deleted_count: provider_counts[3],
            usage: provider_usage,
        },
        bindings,
        catalog,
        trend,
    }))
}

/// The tenant default binding as resolved by the binding APIs.
///
/// `binding_id` is always the deterministic default id; `provider_id` is only
/// populated when a row with that id actually exists and is active, which is
/// exactly the condition `storageProviderBindings.default.retrieve` uses to
/// decide between 200 and 404.
struct TenantDefaultBinding {
    binding_id: String,
    provider_id: Option<String>,
}

struct LoadedCapacity {
    generated_at: String,
    metrics: StorageOverviewCapacityResponse,
}

async fn load_capacity(
    state: &AdminStorageState,
    tenant_id: &str,
) -> Result<LoadedCapacity, (StatusCode, Json<ProblemDetail>)> {
    let object_row = sqlx::query(
        "SELECT
           to_char(NOW() AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"') AS generated_at,
           count(*)::bigint AS total_object_count,
           count(*) FILTER (WHERE lifecycle_status = 'active')::bigint AS active_object_count,
           count(*) FILTER (WHERE lifecycle_status <> 'active')::bigint AS deleted_object_count,
           coalesce(sum(content_length) FILTER (WHERE lifecycle_status = 'active'), 0)::bigint
             AS used_bytes,
           coalesce(round(avg(content_length) FILTER (WHERE lifecycle_status = 'active')), 0)::bigint
             AS average_object_bytes,
           max(content_length) FILTER (WHERE lifecycle_status = 'active')::bigint
             AS largest_object_bytes,
           count(DISTINCT bucket) FILTER (WHERE lifecycle_status = 'active')::bigint
             AS bucket_count
         FROM dr_drive_storage_object
         WHERE tenant_id = $1",
    )
    .bind(tenant_id)
    .fetch_one(&state.pool)
    .await
    .map_err(map_storage_overview_error("capacity"))?;

    let used_bytes = object_row.get::<i64, _>("used_bytes");
    let quota_bytes = load_tenant_quota_bytes(state, tenant_id).await?;
    let quota_usage_ratio = quota_bytes
        .filter(|max_bytes| *max_bytes > 0)
        .map(|max_bytes| used_bytes as f64 / max_bytes as f64);

    Ok(LoadedCapacity {
        generated_at: object_row.get::<String, _>("generated_at"),
        metrics: StorageOverviewCapacityResponse {
            total_object_count: object_row.get::<i64, _>("total_object_count"),
            active_object_count: object_row.get::<i64, _>("active_object_count"),
            deleted_object_count: object_row.get::<i64, _>("deleted_object_count"),
            used_bytes,
            average_object_bytes: object_row.get::<i64, _>("average_object_bytes"),
            largest_object_bytes: object_row.get::<Option<i64>, _>("largest_object_bytes"),
            bucket_count: object_row.get::<i64, _>("bucket_count"),
            quota_configured: quota_bytes.is_some(),
            quota_usage_ratio,
            quota_bytes,
        },
    })
}

/// One tenant has one quota row, and `max_bytes` may legitimately be null for
/// "unlimited". Those are different states: only a non-null limit is a
/// *configured* quota, so usage can be expressed as a ratio.
async fn load_tenant_quota_bytes(
    state: &AdminStorageState,
    tenant_id: &str,
) -> Result<Option<i64>, (StatusCode, Json<ProblemDetail>)> {
    sqlx::query_scalar::<_, Option<i64>>(
        "SELECT max_bytes FROM dr_drive_tenant_quota WHERE tenant_id = $1",
    )
    .bind(tenant_id)
    .fetch_optional(&state.pool)
    .await
    .map(|value| value.flatten())
    .map_err(map_storage_overview_error("tenant quota"))
}

async fn load_provider_counts(
    state: &AdminStorageState,
    tenant_id: &str,
) -> Result<[i64; 4], (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(
        "SELECT
           count(*)::bigint AS total_count,
           count(*) FILTER (WHERE p.status = 'active')::bigint AS active_count,
           count(*) FILTER (WHERE p.status = 'disabled')::bigint AS disabled_count,
           count(*) FILTER (WHERE p.status = 'deleted')::bigint AS deleted_count
         FROM dr_drive_storage_provider p
         WHERE EXISTS (
             SELECT 1 FROM dr_drive_storage_provider_binding pb
             WHERE pb.provider_id = p.id AND pb.tenant_id = $1
           )
           OR EXISTS (
             SELECT 1 FROM dr_drive_storage_object po
             WHERE po.storage_provider_id = p.id AND po.tenant_id = $1
           )",
    )
    .bind(tenant_id)
    .fetch_one(&state.pool)
    .await
    .map_err(map_storage_overview_error("provider counts"))?;

    Ok([
        row.get::<i64, _>("total_count"),
        row.get::<i64, _>("active_count"),
        row.get::<i64, _>("disabled_count"),
        row.get::<i64, _>("deleted_count"),
    ])
}

/// Per-provider usage rows for the providers this tenant depends on.
///
/// The membership predicate mirrors `load_provider_counts` so the headline
/// counts and the rows describe the same set; counting every provider row would
/// mix scopes instead, because `dr_drive_storage_provider` is not tenant
/// filtered in this admin surface.
async fn load_provider_usage(
    state: &AdminStorageState,
    tenant_id: &str,
    total_used_bytes: i64,
    tenant_default: &TenantDefaultBinding,
) -> Result<Vec<StorageOverviewProviderUsageResponse>, (StatusCode, Json<ProblemDetail>)> {
    let rows = sqlx::query(
        "SELECT p.id AS provider_id,
                p.name AS name,
                p.provider_kind AS provider_kind,
                p.status AS status,
                p.bucket AS bucket,
                coalesce(usage.object_count, 0)::bigint AS object_count,
                coalesce(usage.used_bytes, 0)::bigint AS used_bytes,
                coalesce(active_bindings.binding_count, 0)::bigint AS binding_count
         FROM dr_drive_storage_provider p
         LEFT JOIN (
           SELECT storage_provider_id,
                  count(*)::bigint AS object_count,
                  sum(content_length)::bigint AS used_bytes
           FROM dr_drive_storage_object
           WHERE tenant_id = $1 AND lifecycle_status = 'active'
           GROUP BY storage_provider_id
         ) usage ON usage.storage_provider_id = p.id
         LEFT JOIN (
           SELECT provider_id, count(*)::bigint AS binding_count
           FROM dr_drive_storage_provider_binding
           WHERE tenant_id = $1 AND lifecycle_status = 'active'
           GROUP BY provider_id
         ) active_bindings ON active_bindings.provider_id = p.id
         WHERE EXISTS (
             SELECT 1 FROM dr_drive_storage_provider_binding pb
             WHERE pb.provider_id = p.id AND pb.tenant_id = $1
           )
           OR EXISTS (
             SELECT 1 FROM dr_drive_storage_object po
             WHERE po.storage_provider_id = p.id AND po.tenant_id = $1
           )
         ORDER BY coalesce(usage.used_bytes, 0) DESC, p.id ASC",
    )
    .bind(tenant_id)
    .fetch_all(&state.pool)
    .await
    .map_err(map_storage_overview_error("provider usage"))?;

    Ok(rows
        .iter()
        .map(|row| {
            let provider_id = row.get::<String, _>("provider_id");
            let used_bytes = row.get::<i64, _>("used_bytes");
            StorageOverviewProviderUsageResponse {
                is_tenant_default: Some(provider_id.as_str()) == tenant_default.provider_id.as_deref(),
                provider_id,
                name: row.get::<String, _>("name"),
                provider_kind: row.get::<String, _>("provider_kind"),
                status: row.get::<String, _>("status"),
                bucket: row.get::<String, _>("bucket"),
                object_count: row.get::<i64, _>("object_count"),
                used_bytes,
                binding_count: row.get::<i64, _>("binding_count"),
                capacity_share: ratio(used_bytes, total_used_bytes),
            }
        })
        .collect())
}

async fn load_bindings(
    state: &AdminStorageState,
    tenant_id: &str,
    tenant_default: &TenantDefaultBinding,
) -> Result<StorageOverviewBindingsResponse, (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(
        "SELECT
           count(*)::bigint AS total_count,
           count(*) FILTER (WHERE lifecycle_status = 'active')::bigint AS active_count,
           count(*) FILTER (WHERE lifecycle_status <> 'active')::bigint AS inactive_count,
           count(*) FILTER (WHERE binding_scope = 'tenant')::bigint AS tenant_count,
           count(*) FILTER (WHERE binding_scope = 'space')::bigint AS space_count,
           count(*) FILTER (WHERE binding_scope = 'space_type')::bigint AS space_type_count
         FROM dr_drive_storage_provider_binding
         WHERE tenant_id = $1",
    )
    .bind(tenant_id)
    .fetch_one(&state.pool)
    .await
    .map_err(map_storage_overview_error("binding counts"))?;

    Ok(StorageOverviewBindingsResponse {
        total_count: row.get::<i64, _>("total_count"),
        active_count: row.get::<i64, _>("active_count"),
        inactive_count: row.get::<i64, _>("inactive_count"),
        by_scope: StorageOverviewBindingScopeCountsResponse {
            tenant_count: row.get::<i64, _>("tenant_count"),
            space_count: row.get::<i64, _>("space_count"),
            space_type_count: row.get::<i64, _>("space_type_count"),
        },
        has_tenant_default: tenant_default.provider_id.is_some(),
        tenant_default_binding_id: tenant_default
            .provider_id
            .as_ref()
            .map(|_| tenant_default.binding_id.clone()),
        tenant_default_provider_id: tenant_default.provider_id.clone(),
    })
}

/// The tenant default binding, addressed by its deterministic id — the same
/// string `storageProviderBindings.default.retrieve` resolves — so the
/// dashboard cannot disagree with the binding page about whether one exists.
async fn load_tenant_default_binding(
    state: &AdminStorageState,
    tenant_id: &str,
) -> Result<TenantDefaultBinding, (StatusCode, Json<ProblemDetail>)> {
    let binding_id =
        default_storage_provider_binding_id(tenant_id, &StorageProviderBindingTarget::Tenant);
    let provider_id = sqlx::query_scalar::<_, String>(
        "SELECT provider_id
         FROM dr_drive_storage_provider_binding
         WHERE id = $1 AND tenant_id = $2 AND lifecycle_status = 'active'
         LIMIT 1",
    )
    .bind(&binding_id)
    .bind(tenant_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(map_storage_overview_error("tenant default binding"))?;

    Ok(TenantDefaultBinding {
        binding_id,
        provider_id,
    })
}

async fn load_catalog(
    state: &AdminStorageState,
) -> Result<StorageOverviewCatalogResponse, (StatusCode, Json<ProblemDetail>)> {
    let row = sqlx::query(
        "SELECT
           count(*)::bigint AS total_count,
           count(*) FILTER (WHERE enabled)::bigint AS enabled_count,
           count(*) FILTER (WHERE NOT enabled)::bigint AS disabled_count
         FROM dr_drive_storage_provider_kind",
    )
    .fetch_one(&state.pool)
    .await
    .map_err(map_storage_overview_error("provider catalog"))?;

    Ok(StorageOverviewCatalogResponse {
        total_count: row.get::<i64, _>("total_count"),
        enabled_count: row.get::<i64, _>("enabled_count"),
        disabled_count: row.get::<i64, _>("disabled_count"),
    })
}

/// Monthly ingestion trend with explicit zero buckets.
///
/// Without `generate_series` a quiet month would simply be absent from the
/// array and the chart would draw its two neighbours as adjacent columns, which
/// reads as continuous growth when nothing actually happened.
async fn load_trend(
    state: &AdminStorageState,
    tenant_id: &str,
    trend_months: i64,
) -> Result<Vec<StorageOverviewTrendPointResponse>, (StatusCode, Json<ProblemDetail>)> {
    let months = trend_months as i32;
    let rows = sqlx::query(
        "SELECT to_char(month_bucket, 'YYYY-MM') AS period_label,
                coalesce(monthly.object_count, 0)::bigint AS object_count,
                coalesce(monthly.bytes, 0)::bigint AS bytes
         FROM generate_series(
                date_trunc('month', NOW() AT TIME ZONE 'UTC')
                  - (($2::int - 1) * interval '1 month'),
                date_trunc('month', NOW() AT TIME ZONE 'UTC'),
                interval '1 month'
              ) AS month_bucket
         LEFT JOIN (
           SELECT date_trunc('month', created_at AT TIME ZONE 'UTC') AS month_start,
                  count(*)::bigint AS object_count,
                  sum(content_length)::bigint AS bytes
           FROM dr_drive_storage_object
           WHERE tenant_id = $1
             AND created_at >= date_trunc('month', NOW())
               - (($2::int - 1) * interval '1 month')
           GROUP BY 1
         ) monthly ON monthly.month_start = month_bucket
         ORDER BY month_bucket ASC",
    )
    .bind(tenant_id)
    .bind(months)
    .fetch_all(&state.pool)
    .await
    .map_err(map_storage_overview_error("storage trend"))?;

    Ok(rows
        .iter()
        .map(|row| StorageOverviewTrendPointResponse {
            period_label: row.get::<String, _>("period_label"),
            object_count: row.get::<i64, _>("object_count"),
            bytes: row.get::<i64, _>("bytes"),
        })
        .collect())
}

/// Share of `part` in `whole`, 0 when there is nothing to divide by.
fn ratio(part: i64, whole: i64) -> f64 {
    if whole <= 0 {
        return 0.0;
    }
    part as f64 / whole as f64
}

/// Validates the requested trend window.
///
/// Extracted from the handler so the reject/clamp decision is unit-testable:
/// an inline `if` behind an HTTP extension extractor is a blind spot for tests
/// that never build the router.
fn resolve_trend_months(
    requested: Option<i64>,
) -> Result<i64, (StatusCode, Json<ProblemDetail>)> {
    match requested {
        None => Ok(TREND_MONTHS_DEFAULT),
        Some(months) if (TREND_MONTHS_MIN..=TREND_MONTHS_MAX).contains(&months) => Ok(months),
        Some(months) => Err(validation_problem(format!(
            "trendMonths must be between {TREND_MONTHS_MIN} and {TREND_MONTHS_MAX}; got {months}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn error_detail(
        error: Result<i64, (StatusCode, Json<ProblemDetail>)>,
    ) -> (StatusCode, String) {
        let (status, json) = error.expect_err("expected a validation problem");
        // `SdkWorkProblemDetail` fields are crate-private; read the serialized
        // wire form instead, which is the contract the client actually sees.
        let body = serde_json::to_value(json.0).expect("problem detail serializes");
        let detail = body
            .get("detail")
            .and_then(serde_json::Value::as_str)
            .expect("problem detail carries a string detail")
            .to_string();
        (status, detail)
    }

    #[test]
    fn trend_months_defaults_to_twelve_when_absent() {
        assert_eq!(resolve_trend_months(None).expect("default window"), 12);
    }

    #[test]
    fn trend_months_accepts_the_documented_inclusive_bounds() {
        assert_eq!(resolve_trend_months(Some(1)).expect("lower bound"), 1);
        assert_eq!(resolve_trend_months(Some(24)).expect("upper bound"), 24);
    }

    #[test]
    fn trend_months_rejects_out_of_bounds_windows_instead_of_clamping() {
        for months in [0, -1, 25, 100] {
            let (status, detail) = error_detail(resolve_trend_months(Some(months)));
            assert_eq!(status, StatusCode::BAD_REQUEST);
            assert!(detail.contains("trendMonths"), "detail was: {detail}");
        }
    }

    #[test]
    fn ratio_is_zero_without_a_divisible_whole() {
        assert_eq!(ratio(0, 0), 0.0);
        assert_eq!(ratio(5, 0), 0.0);
        assert_eq!(ratio(5, -1), 0.0);
    }

    #[test]
    fn ratio_keeps_partial_shares_and_caps_at_nothing() {
        assert!((ratio(50, 100) - 0.5).abs() < f64::EPSILON);
        assert!((ratio(100, 100) - 1.0).abs() < f64::EPSILON);
        // Overrun is preserved so the UI can flag a busted quota rather than
        // silently normalising to 100%.
        assert!((ratio(150, 100) - 1.5).abs() < f64::EPSILON);
    }
}

fn map_storage_overview_error(
    figure: &'static str,
) -> impl Fn(sqlx::Error) -> (StatusCode, Json<ProblemDetail>) {
    move |error| {
        map_service_error(DriveServiceError::Internal(format!(
            "aggregate storage overview {figure} failed: {error}"
        )))
    }
}
