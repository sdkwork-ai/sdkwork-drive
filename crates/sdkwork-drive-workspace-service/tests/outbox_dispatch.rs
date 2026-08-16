use async_trait::async_trait;
use sdkwork_drive_workspace_service::infrastructure::outbox_dispatch::{
    dispatch_pending_outbox_events, dispatch_pending_outbox_events_with_relay,
};
use sdkwork_drive_workspace_service::ports::domain_outbox_embedded_relay::{
    DeliverDriveDomainOutboxEmbeddedEventRequest, DriveDomainOutboxEmbeddedRelay,
    DriveDomainOutboxEmbeddedRelayError, DriveDomainOutboxEmbeddedTarget,
    ResolveDriveDomainOutboxEmbeddedTargetsRequest,
};
use std::sync::atomic::{AtomicUsize, Ordering};

#[tokio::test]
async fn postgres_claims_pending_outbox_event() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    sqlx::query(
        "INSERT INTO dr_drive_domain_outbox (
            id, tenant_id, space_id, node_id, event_type, actor_id, sequence_no, payload_json
        ) VALUES (
            'outbox-1', '100001', 'space-1', NULL, 'node.updated', 'user-1', 1, '{}'
        )",
    )
    .execute(&pool)
    .await
    .expect("seed outbox row");

    let result = dispatch_pending_outbox_events(&pool)
        .await
        .expect("sqlite outbox dispatch should run");
    assert_eq!(1, result.processed);
    assert_eq!(1, result.delivered);
    assert_eq!(0, result.failed);

    let row: (i32, String) = sqlx::query_as(
        "SELECT attempt_count, delivery_status
         FROM dr_drive_domain_outbox WHERE id = 'outbox-1'",
    )
    .fetch_one(&pool)
    .await
    .expect("outbox row should be completed");
    assert_eq!(1, row.0);
    assert_eq!("delivered", row.1);
}

#[tokio::test]
async fn postgres_embedded_relay_reuses_outbox_retry_and_channel_idempotency() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    sqlx::query(
        "INSERT INTO dr_drive_domain_outbox (
            id, tenant_id, space_id, node_id, event_type, actor_id, sequence_no, payload_json
        ) VALUES (
            'outbox-embedded-1', '100001', 'space-1', 'node-1',
            'drive.node.deleted.v1', 'user-1', 1, '{\"id\":\"event-embedded-1\"}'
        )",
    )
    .execute(&pool)
    .await
    .expect("seed embedded outbox row");
    let relay = RecordingRelay::default();

    let first = dispatch_pending_outbox_events_with_relay(&pool, &relay)
        .await
        .expect("embedded outbox dispatch should run");
    assert_eq!(first.processed, 1);
    assert_eq!(first.delivered, 1);
    assert_eq!(relay.deliveries.load(Ordering::Relaxed), 1);

    let status: String = sqlx::query_scalar(
        "SELECT delivery_status FROM dr_drive_domain_outbox WHERE id = 'outbox-embedded-1'",
    )
    .fetch_one(&pool)
    .await
    .expect("read delivered outbox status");
    assert_eq!(status, "delivered");
    let channel_deliveries: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM dr_drive_domain_outbox_channel_delivery
         WHERE outbox_id = 'outbox-embedded-1' AND channel_id = 'embedded:kbraw:scope-1'",
    )
    .fetch_one(&pool)
    .await
    .expect("read embedded channel delivery");
    assert_eq!(channel_deliveries, 1);

    let replay = dispatch_pending_outbox_events_with_relay(&pool, &relay)
        .await
        .expect("delivered outbox replay should be idle");
    assert_eq!(replay.processed, 0);
    assert_eq!(relay.deliveries.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn postgres_does_not_contact_an_unmatched_provider_root_channel() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    sqlx::query(
        "INSERT INTO dr_drive_space (
            id, tenant_id, owner_subject_type, owner_subject_id, space_type,
            display_name, lifecycle_status, version, created_by, updated_by
         ) VALUES (
            'space-web', 'tenant-web', 'organization', 'organization-web', 'website',
            'Website', 'active', 1, 'test', 'test'
         )",
    )
    .execute(&pool)
    .await
    .expect("seed website Space");
    sqlx::query(
        "INSERT INTO dr_drive_watch_channel (
            id, tenant_id, space_id, node_id, resource_type, resource_id,
            channel_type, address, token_hash, expiration_epoch_ms,
            lifecycle_status, version, created_by, updated_by
         ) VALUES (
            'web-node-other-root', 'tenant-web', 'space-web', NULL, 'changes',
            'website-root-other', 'web_hook', 'https://127.0.0.1:9/must-not-be-called',
            'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
            1999999999999, 'active', 1, 'test', 'test'
         )",
    )
    .execute(&pool)
    .await
    .expect("seed unmatched provider root channel");
    sqlx::query(
        "INSERT INTO dr_drive_domain_outbox (
            id, tenant_id, space_id, node_id, event_type, actor_id, sequence_no, payload_json
         ) VALUES (
            'outbox-root-routing-1', 'tenant-web', 'space-web', NULL,
            'drive.website_root.generation.changed.v1', 'user-1', 1,
            '{\"id\":\"event-root-routing-1\",\"data\":{\"websiteRootUuid\":\"website-root-matching\"}}'
         )",
    )
    .execute(&pool)
    .await
    .expect("seed website root generation event");

    let result = dispatch_pending_outbox_events(&pool)
        .await
        .expect("unmatched root channel must be filtered before network delivery");
    assert_eq!(result.processed, 1);
    assert_eq!(result.delivered, 1);
    assert_eq!(result.failed, 0);

    let outbox_status: String = sqlx::query_scalar(
        "SELECT delivery_status FROM dr_drive_domain_outbox
         WHERE id='outbox-root-routing-1'",
    )
    .fetch_one(&pool)
    .await
    .expect("read routed outbox status");
    assert_eq!(outbox_status, "delivered");
    let channel_delivery_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM dr_drive_domain_outbox_channel_delivery
         WHERE outbox_id='outbox-root-routing-1'",
    )
    .fetch_one(&pool)
    .await
    .expect("read unmatched channel delivery count");
    assert_eq!(channel_delivery_count, 0);
}

#[tokio::test]
async fn postgres_claims_pending_outbox_event_with_skip_locked() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let outbox_id = format!("outbox-pg-{}", uuid::Uuid::new_v4());
    sqlx::query(
        "INSERT INTO dr_drive_domain_outbox (
            id, tenant_id, space_id, node_id, event_type, actor_id, sequence_no, payload_json,
            created_at
        ) VALUES (
            $1, '100001', 'space-1', NULL, 'node.updated', 'user-1', 1, '{}',
            TIMESTAMPTZ '2000-01-01 00:00:00+00'
        )",
    )
    .bind(&outbox_id)
    .execute(&pool)
    .await
    .expect("seed postgres outbox row");

    let result = dispatch_pending_outbox_events(&pool)
        .await
        .expect("postgres outbox dispatch should run");
    assert!(
        result.processed >= 1,
        "dispatcher must process the seeded event even when the contract database contains other pending events"
    );

    let row: (i32, String) = sqlx::query_as(
        "SELECT attempt_count, delivery_status
         FROM dr_drive_domain_outbox WHERE id = $1",
    )
    .bind(&outbox_id)
    .fetch_one(&pool)
    .await
    .expect("postgres outbox row should complete");
    assert_eq!(1, row.0);
    assert_eq!("delivered", row.1);
}

#[derive(Default)]
struct RecordingRelay {
    deliveries: AtomicUsize,
}

#[async_trait]
impl DriveDomainOutboxEmbeddedRelay for RecordingRelay {
    async fn resolve_targets(
        &self,
        request: ResolveDriveDomainOutboxEmbeddedTargetsRequest<'_>,
    ) -> Result<Vec<DriveDomainOutboxEmbeddedTarget>, DriveDomainOutboxEmbeddedRelayError> {
        assert_eq!(request.tenant_id, "100001");
        assert_eq!(request.space_id, "space-1");
        Ok(vec![DriveDomainOutboxEmbeddedTarget {
            channel_id: "embedded:kbraw:scope-1".to_string(),
            source_scope_uuid: "scope-1".to_string(),
        }])
    }

    async fn deliver(
        &self,
        request: DeliverDriveDomainOutboxEmbeddedEventRequest<'_>,
    ) -> Result<(), DriveDomainOutboxEmbeddedRelayError> {
        assert_eq!(request.outbox_id, "outbox-embedded-1");
        assert_eq!(request.channel_id, "embedded:kbraw:scope-1");
        assert_eq!(request.source_scope_uuid, "scope-1");
        self.deliveries.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}
