use crate::ports::domain_outbox_embedded_relay::{
    DeliverDriveDomainOutboxEmbeddedEventRequest, DriveDomainOutboxEmbeddedRelay,
    DriveDomainOutboxEmbeddedTarget, ResolveDriveDomainOutboxEmbeddedTargetsRequest,
    MAX_EMBEDDED_OUTBOX_TARGETS_PER_EVENT,
};
use sdkwork_drive_contract::drive::events::{
    sign_webhook, WEBHOOK_CHANNEL_ID_HEADER, WEBHOOK_EVENT_ID_HEADER,
    WEBHOOK_EVENT_RETRY_COUNT_HEADER, WEBHOOK_EVENT_SIGNATURE_HEADER,
    WEBHOOK_EVENT_TIMESTAMP_HEADER, WEBHOOK_IDEMPOTENCY_KEY_HEADER,
};
use sdkwork_drive_observability::metrics;
use serde_json::Value;
use sqlx::{PgPool, Row};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};

const MAX_OUTBOX_ATTEMPTS: i32 = 10;
const OUTBOX_BATCH_SIZE: i64 = 100;
const KNOWLEDGEBASE_EVENT_INGRESS_TOKEN_FILE_ENV: &str =
    "SDKWORK_DRIVE_KNOWLEDGEBASE_EVENT_INGRESS_TOKEN_FILE";
const KNOWLEDGEBASE_EVENT_CALLBACK_URL_ENV: &str = "SDKWORK_DRIVE_KNOWLEDGEBASE_EVENT_CALLBACK_URL";
/// Bound webhook fan-out per outbox event to keep dispatch memory O(channels cap).
const MAX_WATCH_CHANNELS_PER_OUTBOX_EVENT: i64 = 200;
const MAX_PROVIDER_SCOPES_PER_OUTBOX_EVENT: usize = 512;

static DOMAIN_OUTBOX_DISPATCHER_STARTED: AtomicBool = AtomicBool::new(false);
static IMMEDIATE_OUTBOX_DISPATCH_IN_FLIGHT: AtomicBool = AtomicBool::new(false);

fn embedded_outbox_dispatch_disabled() -> bool {
    std::env::var("SDKWORK_DRIVE_DOMAIN_OUTBOX_EMBEDDED_DISPATCH")
        .ok()
        .map(|value| {
            let value = value.trim().to_ascii_lowercase();
            value == "0" || value == "false" || value == "off" || value == "disabled"
        })
        .unwrap_or(false)
}

#[derive(Debug, Clone, Default)]
pub struct DomainOutboxDispatchResult {
    pub processed: usize,
    pub delivered: usize,
    pub failed: usize,
}

/// Starts at most one embedded periodic outbox dispatcher per process.
///
/// Cloud deployments that run `sdkwork-drive-install-worker` should set
/// `SDKWORK_DRIVE_DOMAIN_OUTBOX_EMBEDDED_DISPATCH=false` on API pods.
pub fn ensure_domain_outbox_dispatcher(pool: PgPool) {
    if embedded_outbox_dispatch_disabled() {
        return;
    }
    if DOMAIN_OUTBOX_DISPATCHER_STARTED
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }

    tokio::spawn(async move {
        let interval_secs = std::env::var("SDKWORK_DRIVE_DOMAIN_OUTBOX_DISPATCH_INTERVAL_SECONDS")
            .ok()
            .and_then(|value| value.trim().parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(15);
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
        interval.tick().await;
        loop {
            interval.tick().await;
            if let Err(error) = dispatch_pending_outbox_events(&pool).await {
                tracing::warn!(
                    target: "sdkwork.drive",
                    event = "drive.domain_outbox.dispatch_failed",
                    error = %error,
                    "domain outbox dispatch failed"
                );
            }
        }
    });
}

/// Backward-compatible alias for callers that previously spawned duplicate loops.
pub fn spawn_pending_outbox_dispatch(pool: PgPool) {
    ensure_domain_outbox_dispatcher(pool);
}

/// Attempt one immediate outbox delivery pass without starting another periodic loop.
pub fn trigger_immediate_outbox_dispatch(pool: PgPool) {
    if embedded_outbox_dispatch_disabled() {
        return;
    }
    if IMMEDIATE_OUTBOX_DISPATCH_IN_FLIGHT
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }

    tokio::spawn(async move {
        let dispatch_result = dispatch_pending_outbox_events(&pool).await;
        IMMEDIATE_OUTBOX_DISPATCH_IN_FLIGHT.store(false, Ordering::Release);
        if let Err(error) = dispatch_result {
            tracing::warn!(
                target: "sdkwork.drive",
                event = "drive.domain_outbox.immediate_dispatch_failed",
                error = %error,
                "immediate domain outbox dispatch failed"
            );
        }
    });
}

pub async fn dispatch_pending_outbox_events(
    pool: &PgPool,
) -> Result<DomainOutboxDispatchResult, String> {
    dispatch_pending_outbox_events_with_optional_relay(pool, None).await
}

pub async fn dispatch_pending_outbox_events_with_relay(
    pool: &PgPool,
    relay: &dyn DriveDomainOutboxEmbeddedRelay,
) -> Result<DomainOutboxDispatchResult, String> {
    dispatch_pending_outbox_events_with_optional_relay(pool, Some(relay)).await
}

async fn dispatch_pending_outbox_events_with_optional_relay(
    pool: &PgPool,
    relay: Option<&dyn DriveDomainOutboxEmbeddedRelay>,
) -> Result<DomainOutboxDispatchResult, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|error| format!("build outbox webhook client failed: {error}"))?;

    let mut result = DomainOutboxDispatchResult::default();
    let mut processed_in_batch = HashSet::new();
    for _ in 0..OUTBOX_BATCH_SIZE {
        let Some(claimed) = claim_next_pending_outbox_event(pool, &processed_in_batch).await?
        else {
            break;
        };
        processed_in_batch.insert(claimed.outbox_id.clone());

        result.processed += 1;
        if claimed.attempt_count > MAX_OUTBOX_ATTEMPTS {
            mark_outbox_event_failed(
                pool,
                &claimed.outbox_id,
                claimed.attempt_count,
                "outbox delivery attempts exhausted during claim",
            )
            .await?;
            result.failed += 1;
            continue;
        }

        match dispatch_outbox_event(
            pool,
            &client,
            relay,
            &claimed.outbox_id,
            &OutboxChangeEvent {
                tenant_id: &claimed.tenant_id,
                space_id: &claimed.space_id,
                event_type: &claimed.event_type,
                attempt_count: claimed.attempt_count,
                payload_json: &claimed.payload_json,
            },
        )
        .await
        {
            Ok(()) => {
                mark_outbox_event_delivered(pool, &claimed.outbox_id).await?;
                metrics::record_outbox_delivered();
                result.delivered += 1;
            }
            Err(error) => {
                let delivery_status = if claimed.attempt_count >= MAX_OUTBOX_ATTEMPTS {
                    "failed"
                } else {
                    "pending"
                };
                mark_outbox_event_attempt_failed(
                    pool,
                    &claimed.outbox_id,
                    claimed.attempt_count,
                    delivery_status,
                    &error,
                )
                .await?;
                if delivery_status == "failed" {
                    result.failed += 1;
                }
            }
        }
    }

    Ok(result)
}

struct ClaimedOutboxEvent {
    outbox_id: String,
    tenant_id: String,
    space_id: String,
    event_type: String,
    attempt_count: i32,
    payload_json: String,
}

async fn claim_next_pending_outbox_event(
    pool: &PgPool,
    exclude_ids: &HashSet<String>,
) -> Result<Option<ClaimedOutboxEvent>, String> {
    let exclude_clause = build_outbox_claim_exclude_clause(exclude_ids);
    let row = sqlx::query(sqlx::AssertSqlSafe(format!(
        "UPDATE dr_drive_domain_outbox
         SET attempt_count = attempt_count + 1
         WHERE id = (
           SELECT id
           FROM dr_drive_domain_outbox
           WHERE delivery_status = 'pending' AND attempt_count < $1{exclude_clause}
           ORDER BY created_at ASC
           LIMIT 1
           FOR UPDATE SKIP LOCKED
         )
         RETURNING id, tenant_id, space_id, event_type, attempt_count, payload_json",
    )))
    .bind(MAX_OUTBOX_ATTEMPTS)
    .fetch_optional(pool)
    .await
    .map_err(|error| format!("claim pending domain outbox event failed: {error}"))?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(ClaimedOutboxEvent {
        outbox_id: row.get("id"),
        tenant_id: row.get("tenant_id"),
        space_id: row.get("space_id"),
        event_type: row.get("event_type"),
        attempt_count: row.get("attempt_count"),
        payload_json: row.get("payload_json"),
    }))
}

async fn mark_outbox_event_delivered(pool: &PgPool, outbox_id: &str) -> Result<(), String> {
    sqlx::query(
        "UPDATE dr_drive_domain_outbox
         SET delivery_status = 'delivered',
             delivered_at = CURRENT_TIMESTAMP,
             last_error = NULL
         WHERE id = $1",
    )
    .bind(outbox_id)
    .execute(pool)
    .await
    .map_err(|error| format!("mark domain outbox event delivered failed: {error}"))?;
    Ok(())
}

async fn mark_outbox_event_attempt_failed(
    pool: &PgPool,
    outbox_id: &str,
    attempt_count: i32,
    delivery_status: &str,
    error: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE dr_drive_domain_outbox
         SET delivery_status = $1,
             last_error = $2
         WHERE id = $3 AND attempt_count = $4",
    )
    .bind(delivery_status)
    .bind(error)
    .bind(outbox_id)
    .bind(attempt_count)
    .execute(pool)
    .await
    .map_err(|update_error| format!("update failed domain outbox event failed: {update_error}"))?;
    Ok(())
}

async fn mark_outbox_event_failed(
    pool: &PgPool,
    outbox_id: &str,
    attempt_count: i32,
    error: &str,
) -> Result<(), String> {
    mark_outbox_event_attempt_failed(pool, outbox_id, attempt_count, "failed", error).await
}

struct OutboxChangeEvent<'a> {
    tenant_id: &'a str,
    space_id: &'a str,
    event_type: &'a str,
    attempt_count: i32,
    payload_json: &'a str,
}

async fn dispatch_outbox_event(
    pool: &PgPool,
    client: &reqwest::Client,
    relay: Option<&dyn DriveDomainOutboxEmbeddedRelay>,
    outbox_id: &str,
    event: &OutboxChangeEvent<'_>,
) -> Result<(), String> {
    let now_epoch_ms = chrono::Utc::now().timestamp_millis();
    let payload: Value = serde_json::from_str(event.payload_json)
        .map_err(|error| format!("domain outbox payload is invalid JSON: {error}"))?;
    let affected_resources = affected_provider_resource_ids(event.event_type, &payload)?;
    let affected_resource_ids = affected_resources.iter().collect::<Vec<_>>();
    let mut channel_query = String::from(
        "SELECT id, address, token_hash, resource_id
         FROM dr_drive_watch_channel
         WHERE tenant_id=$1
           AND lifecycle_status='active'
           AND resource_type='changes'
           AND (space_id IS NULL OR space_id=$2)
           AND expiration_epoch_ms > $3
           AND (resource_id IS NULL OR resource_id=$4",
    );
    for bind_index in 0..affected_resource_ids.len() {
        channel_query.push_str(&format!(" OR resource_id=${}", bind_index + 5));
    }
    channel_query.push_str(&format!(
        ") ORDER BY created_at ASC LIMIT {}",
        MAX_WATCH_CHANNELS_PER_OUTBOX_EVENT + 1
    ));
    let mut query = sqlx::query(sqlx::AssertSqlSafe(channel_query.as_str()))
        .bind(event.tenant_id)
        .bind(event.space_id)
        .bind(now_epoch_ms)
        .bind(event.space_id);
    for resource_id in affected_resource_ids {
        query = query.bind(resource_id);
    }
    let rows = query
        .fetch_all(pool)
        .await
        .map_err(|error| format!("list watch channels for outbox failed: {error}"))?;
    if rows.len() > MAX_WATCH_CHANNELS_PER_OUTBOX_EVENT as usize {
        return Err("active watch channel fan-out exceeds the bounded limit".to_string());
    }
    let rows = rows
        .into_iter()
        .filter(|row| {
            let channel_id: String = row.get("id");
            let resource_id: Option<String> = row.get("resource_id");
            channel_matches_provider_resources(
                &channel_id,
                resource_id.as_deref(),
                event.space_id,
                &affected_resources,
            )
        })
        .collect::<Vec<_>>();
    let timestamp = chrono::Utc::now().timestamp().to_string();
    let retry_count = event.attempt_count.saturating_sub(1).to_string();
    let embedded_targets = resolve_embedded_targets(relay, event).await?;

    if rows.is_empty() && embedded_targets.is_empty() {
        return Ok(());
    }
    let event_id = payload
        .get("id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty() && value.len() <= 128)
        .unwrap_or(outbox_id);

    if let Some(relay) = relay {
        for target in embedded_targets {
            if outbox_channel_already_delivered(pool, outbox_id, &target.channel_id).await? {
                continue;
            }
            relay
                .deliver(DeliverDriveDomainOutboxEmbeddedEventRequest {
                    outbox_id,
                    tenant_id: event.tenant_id,
                    space_id: event.space_id,
                    attempt_count: event.attempt_count,
                    channel_id: &target.channel_id,
                    source_scope_uuid: &target.source_scope_uuid,
                    payload_json: event.payload_json,
                })
                .await
                .map_err(|error| {
                    format!(
                        "dispatch outbox event to embedded channel {} failed: {error}",
                        target.channel_id
                    )
                })?;
            record_outbox_channel_delivery(pool, outbox_id, &target.channel_id).await?;
        }
    }

    for row in rows {
        let channel_id: String = row.get("id");
        let address: String = row.get("address");
        let signing_key: Option<String> = row.get("token_hash");
        let signing_key = signing_key
            .filter(|value| is_sha256_hex(value))
            .ok_or_else(|| format!("outbox webhook channel {channel_id} has no signing key"))?;

        if outbox_channel_already_delivered(pool, outbox_id, &channel_id).await? {
            continue;
        }

        sdkwork_drive_security::validate_webhook_https_url_for_dispatch(&address)
            .await
            .map_err(|detail| {
                format!("outbox webhook channel {channel_id} has invalid address: {detail}")
            })?;
        let idempotency_key = format!("{outbox_id}:{channel_id}");
        let signature = sign_webhook(
            &timestamp,
            event.payload_json.as_bytes(),
            signing_key.as_bytes(),
        );
        let knowledgebase_ingress_token = if channel_id.starts_with("kbraw:") {
            require_configured_knowledgebase_callback(&address)?;
            Some(read_knowledgebase_event_ingress_token()?)
        } else {
            None
        };
        let request = build_webhook_request(
            client,
            &address,
            event_id,
            &timestamp,
            &signature,
            &retry_count,
            &channel_id,
            &idempotency_key,
            event.payload_json.as_bytes(),
            knowledgebase_ingress_token.as_deref(),
        )?;
        let response = client.execute(request).await.map_err(|error| {
            format!("dispatch outbox event to channel {channel_id} failed: {error}")
        })?;
        if !response.status().is_success() {
            return Err(format!(
                "outbox webhook channel {channel_id} returned status {}",
                response.status()
            ));
        }

        record_outbox_channel_delivery(pool, outbox_id, &channel_id).await?;
    }

    Ok(())
}

fn affected_provider_resource_ids(
    event_type: &str,
    payload: &Value,
) -> Result<HashSet<String>, String> {
    let mut resources = HashSet::new();
    let Some(data) = payload.get("data").and_then(Value::as_object) else {
        return Ok(resources);
    };
    match event_type {
        "drive.website_root.generation.changed.v1" => {
            let resource_id = data
                .get("websiteRootUuid")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    "website root generation event has no provider resource UUID".to_string()
                })?;
            insert_provider_resource_id(&mut resources, resource_id)?;
        }
        "drive.node.version.committed.v1"
        | "drive.node.eligibility.changed.v1"
        | "drive.node.deleted.v1" => {
            collect_root_scope_ids(data.get("rootScopes"), &mut resources)?;
        }
        "drive.node.path.changed.v1" => {
            collect_root_scope_ids(data.get("oldRootScopes"), &mut resources)?;
            collect_root_scope_ids(data.get("newRootScopes"), &mut resources)?;
        }
        _ => {}
    }
    Ok(resources)
}

fn collect_root_scope_ids(
    value: Option<&Value>,
    resources: &mut HashSet<String>,
) -> Result<(), String> {
    let scopes = value
        .and_then(Value::as_array)
        .ok_or_else(|| "typed Drive node event has no root scope array".to_string())?;
    for scope in scopes {
        let resource_id = scope
            .get("scopeId")
            .and_then(Value::as_str)
            .ok_or_else(|| "typed Drive node event has an invalid root scope".to_string())?;
        insert_provider_resource_id(resources, resource_id)?;
    }
    Ok(())
}

fn insert_provider_resource_id(
    resources: &mut HashSet<String>,
    resource_id: &str,
) -> Result<(), String> {
    if resource_id.is_empty()
        || resource_id.len() > 64
        || resource_id.trim() != resource_id
        || resource_id.bytes().any(|byte| byte.is_ascii_control())
        || (!resources.contains(resource_id)
            && resources.len() >= MAX_PROVIDER_SCOPES_PER_OUTBOX_EVENT)
    {
        return Err("provider resource scope is outside bounded limits".to_string());
    }
    resources.insert(resource_id.to_string());
    Ok(())
}

fn channel_matches_provider_resources(
    channel_id: &str,
    resource_id: Option<&str>,
    space_id: &str,
    affected_resources: &HashSet<String>,
) -> bool {
    if let Some(root_scope_uuid) = channel_id.strip_prefix("kbraw:") {
        return affected_resources.contains(root_scope_uuid);
    }
    match resource_id {
        Some(resource_id) if resource_id != space_id => affected_resources.contains(resource_id),
        _ => true,
    }
}

async fn resolve_embedded_targets(
    relay: Option<&dyn DriveDomainOutboxEmbeddedRelay>,
    event: &OutboxChangeEvent<'_>,
) -> Result<Vec<DriveDomainOutboxEmbeddedTarget>, String> {
    let Some(relay) = relay else {
        return Ok(Vec::new());
    };
    let targets = relay
        .resolve_targets(ResolveDriveDomainOutboxEmbeddedTargetsRequest {
            tenant_id: event.tenant_id,
            space_id: event.space_id,
            payload_json: event.payload_json,
        })
        .await
        .map_err(|error| format!("resolve embedded outbox targets failed: {error}"))?;
    validate_embedded_targets(targets)
}

fn validate_embedded_targets(
    targets: Vec<DriveDomainOutboxEmbeddedTarget>,
) -> Result<Vec<DriveDomainOutboxEmbeddedTarget>, String> {
    if targets.len() > MAX_EMBEDDED_OUTBOX_TARGETS_PER_EVENT {
        return Err("embedded outbox target count exceeds the bounded fan-out limit".to_string());
    }
    let mut unique = HashMap::with_capacity(targets.len());
    for target in targets {
        if target.channel_id.is_empty()
            || target.channel_id.len() > 200
            || target.channel_id.chars().any(char::is_whitespace)
            || target.source_scope_uuid.is_empty()
            || target.source_scope_uuid.len() > 160
            || target.source_scope_uuid.trim() != target.source_scope_uuid
        {
            return Err("embedded outbox target is outside bounded limits".to_string());
        }
        match unique.get(&target.channel_id) {
            Some(source_scope_uuid) if source_scope_uuid != &target.source_scope_uuid => {
                return Err("embedded outbox channel maps to multiple source scopes".to_string());
            }
            Some(_) => {}
            None => {
                unique.insert(target.channel_id.clone(), target.source_scope_uuid.clone());
            }
        }
    }
    Ok(unique
        .into_iter()
        .map(
            |(channel_id, source_scope_uuid)| DriveDomainOutboxEmbeddedTarget {
                channel_id,
                source_scope_uuid,
            },
        )
        .collect())
}

#[allow(clippy::too_many_arguments)]
fn build_webhook_request(
    client: &reqwest::Client,
    address: &str,
    event_id: &str,
    timestamp: &str,
    signature: &str,
    retry_count: &str,
    channel_id: &str,
    idempotency_key: &str,
    body: &[u8],
    knowledgebase_ingress_token: Option<&str>,
) -> Result<reqwest::Request, String> {
    let mut request = client
        .post(address)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(WEBHOOK_EVENT_ID_HEADER, event_id)
        .header(WEBHOOK_EVENT_TIMESTAMP_HEADER, timestamp)
        .header(WEBHOOK_EVENT_SIGNATURE_HEADER, signature)
        .header(WEBHOOK_EVENT_RETRY_COUNT_HEADER, retry_count)
        .header(WEBHOOK_CHANNEL_ID_HEADER, channel_id)
        .header(WEBHOOK_IDEMPOTENCY_KEY_HEADER, idempotency_key)
        .body(body.to_vec());
    if let Some(token) = knowledgebase_ingress_token {
        request = request.header(reqwest::header::HeaderName::from_static("x-api-key"), token);
    }
    request
        .build()
        .map_err(|error| format!("build outbox webhook request failed: {error}"))
}

fn require_configured_knowledgebase_callback(address: &str) -> Result<(), String> {
    let configured = std::env::var(KNOWLEDGEBASE_EVENT_CALLBACK_URL_ENV).map_err(|_| {
        format!("{KNOWLEDGEBASE_EVENT_CALLBACK_URL_ENV} is required for kbraw delivery")
    })?;
    validate_configured_knowledgebase_callback(address, &configured)
}

fn validate_configured_knowledgebase_callback(
    address: &str,
    configured: &str,
) -> Result<(), String> {
    if configured.is_empty()
        || configured.trim() != configured
        || !configured.starts_with("https://")
        || configured != address
    {
        return Err(format!(
            "kbraw delivery address does not match {KNOWLEDGEBASE_EVENT_CALLBACK_URL_ENV}"
        ));
    }
    Ok(())
}

fn read_knowledgebase_event_ingress_token() -> Result<String, String> {
    let path = std::env::var(KNOWLEDGEBASE_EVENT_INGRESS_TOKEN_FILE_ENV).map_err(|_| {
        format!("{KNOWLEDGEBASE_EVENT_INGRESS_TOKEN_FILE_ENV} is required for kbraw delivery")
    })?;
    if path.trim().is_empty() {
        return Err(format!(
            "{KNOWLEDGEBASE_EVENT_INGRESS_TOKEN_FILE_ENV} must not be blank"
        ));
    }
    let metadata = std::fs::metadata(path.trim()).map_err(|_| {
        format!(
            "{KNOWLEDGEBASE_EVENT_INGRESS_TOKEN_FILE_ENV} must reference a readable secret file"
        )
    })?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > 16 * 1024 {
        return Err(format!(
            "{KNOWLEDGEBASE_EVENT_INGRESS_TOKEN_FILE_ENV} must reference a bounded secret file"
        ));
    }
    let token = std::fs::read_to_string(path.trim()).map_err(|_| {
        format!("{KNOWLEDGEBASE_EVENT_INGRESS_TOKEN_FILE_ENV} must reference a UTF-8 secret file")
    })?;
    if token.len() < 16 || token.len() > 4_096 || token.chars().any(char::is_whitespace) {
        return Err(format!(
            "{KNOWLEDGEBASE_EVENT_INGRESS_TOKEN_FILE_ENV} contains an invalid ingress token"
        ));
    }
    Ok(token)
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

async fn outbox_channel_already_delivered(
    pool: &PgPool,
    outbox_id: &str,
    channel_id: &str,
) -> Result<bool, String> {
    let exists = sqlx::query_scalar::<_, i32>(
        "SELECT 1
         FROM dr_drive_domain_outbox_channel_delivery
         WHERE outbox_id = $1 AND channel_id = $2
         LIMIT 1",
    )
    .bind(outbox_id)
    .bind(channel_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| format!("probe outbox channel delivery failed: {error}"))?;
    Ok(exists.is_some())
}

async fn record_outbox_channel_delivery(
    pool: &PgPool,
    outbox_id: &str,
    channel_id: &str,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO dr_drive_domain_outbox_channel_delivery (outbox_id, channel_id, delivered_at)
         VALUES ($1, $2, CURRENT_TIMESTAMP)
         ON CONFLICT (outbox_id, channel_id) DO NOTHING",
    )
    .bind(outbox_id)
    .bind(channel_id)
    .execute(pool)
    .await
    .map_err(|error| format!("record outbox channel delivery failed: {error}"))?;
    Ok(())
}

fn build_outbox_claim_exclude_clause(exclude_ids: &HashSet<String>) -> String {
    if exclude_ids.is_empty() {
        return String::new();
    }
    let quoted = exclude_ids
        .iter()
        .map(|id| format!("'{}'", id.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(", ");
    format!(" AND id NOT IN ({quoted})")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_outbox_dispatch_disabled_parses_false_values() {
        std::env::set_var("SDKWORK_DRIVE_DOMAIN_OUTBOX_EMBEDDED_DISPATCH", "false");
        assert!(embedded_outbox_dispatch_disabled());
        std::env::remove_var("SDKWORK_DRIVE_DOMAIN_OUTBOX_EMBEDDED_DISPATCH");
    }

    #[test]
    fn outbox_claim_exclude_clause_escapes_single_quotes() {
        let mut exclude = HashSet::new();
        exclude.insert("outbox-'quoted".to_string());
        assert_eq!(
            build_outbox_claim_exclude_clause(&exclude),
            " AND id NOT IN ('outbox-''quoted')"
        );
    }

    #[test]
    fn kbraw_request_carries_ingress_token_and_exact_signed_body() {
        let client = reqwest::Client::new();
        let body = br#"{"specversion":"1.0","id":"event-1"}"#;
        let request = build_webhook_request(
            &client,
            "https://knowledgebase.example.com/internal/v3/api/knowledgebase/drive_events",
            "event-1",
            "1784600000",
            "v1=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "2",
            "kbraw:11111111-1111-4111-8111-111111111111",
            "outbox-1:kbraw:11111111-1111-4111-8111-111111111111",
            body,
            Some("api_key_id=drive-events;tenant_id=100001;app_id=sdkwork-drive"),
        )
        .expect("kbraw request should build");

        assert_eq!(request.method(), reqwest::Method::POST);
        assert_eq!(
            request.url().as_str(),
            "https://knowledgebase.example.com/internal/v3/api/knowledgebase/drive_events"
        );
        assert_eq!(request.headers()["x-sdkwork-event-id"], "event-1");
        assert_eq!(request.headers()["x-sdkwork-event-retry-count"], "2");
        assert_eq!(
            request.headers()["x-sdkwork-drive-channel-id"],
            "kbraw:11111111-1111-4111-8111-111111111111"
        );
        assert_eq!(
            request.headers()["x-api-key"],
            "api_key_id=drive-events;tenant_id=100001;app_id=sdkwork-drive"
        );
        assert_eq!(
            request.body().and_then(reqwest::Body::as_bytes),
            Some(body.as_slice())
        );
    }

    #[test]
    fn ordinary_webhook_request_does_not_carry_internal_ingress_token() {
        let request = build_webhook_request(
            &reqwest::Client::new(),
            "https://hooks.example.com/drive/events",
            "event-1",
            "1784600000",
            "v1=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "0",
            "user-channel-1",
            "outbox-1:user-channel-1",
            br#"{"id":"event-1"}"#,
            None,
        )
        .expect("ordinary webhook request should build");

        assert!(!request.headers().contains_key("x-api-key"));
    }

    #[test]
    fn provider_resource_routing_is_root_scoped_and_keeps_space_channels_generic() {
        let payload = serde_json::json!({
            "data": {
                "oldRootScopes": [
                    {"scopeId": "website-root-1"},
                    {"scopeId": "raw-scope-1"}
                ],
                "newRootScopes": [
                    {"scopeId": "website-root-2"}
                ]
            }
        });
        let resources = affected_provider_resource_ids("drive.node.path.changed.v1", &payload)
            .expect("root scope routing should parse");
        assert_eq!(resources.len(), 3);
        assert!(channel_matches_provider_resources(
            "website-node-1",
            Some("website-root-2"),
            "space-1",
            &resources,
        ));
        assert!(!channel_matches_provider_resources(
            "website-node-1",
            Some("website-root-other"),
            "space-1",
            &resources,
        ));
        assert!(channel_matches_provider_resources(
            "kbraw:raw-scope-1",
            Some("space-1"),
            "space-1",
            &resources,
        ));
        assert!(channel_matches_provider_resources(
            "ordinary-space-channel",
            Some("space-1"),
            "space-1",
            &resources,
        ));
    }

    #[test]
    fn kbraw_callback_must_equal_the_configured_application_ingress_url() {
        assert!(validate_configured_knowledgebase_callback(
            "https://knowledgebase.example.com/internal/v3/api/knowledgebase/drive_events",
            "https://knowledgebase.example.com/internal/v3/api/knowledgebase/drive_events",
        )
        .is_ok());
        assert!(validate_configured_knowledgebase_callback(
            "https://attacker.example.com/events",
            "https://knowledgebase.example.com/internal/v3/api/knowledgebase/drive_events",
        )
        .is_err());
    }
}
