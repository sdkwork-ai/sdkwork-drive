use sqlx::Row;

#[tokio::test]
async fn audit_event_query_plan_uses_filter_indexes_for_list_and_count_patterns() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    // A realistic audit table: many rows, each probe matching only a small
    // fraction. With a handful of rows PostgreSQL correctly prefers a
    // sequential scan for everything, so a small seed cannot distinguish an
    // index-backed plan from a scan.
    for index in 0..5_000 {
        // One action is rare (~1%) and one common (~99%), so the equality probe
        // is selective enough for the planner to prefer the index.
        let action = if index % 100 == 0 {
            "drive.storage_provider.secret_rotated"
        } else {
            "drive.storage_provider.updated"
        };
        let request_id = if index % 500 == 0 {
            "request-rare".to_string()
        } else {
            format!("request-{index:05}")
        };
        let trace_id = if index % 500 == 0 {
            "trace-rare".to_string()
        } else {
            format!("trace-{index:05}")
        };
        sqlx::query(
            "INSERT INTO dr_drive_audit_event (
                id, tenant_id, action, resource_type, resource_id, operator_id, request_id, trace_id
            ) VALUES ($1, $2, $3, 'storage_provider', $4, 'admin-001', $5, $6)",
        )
        .bind(10_000_i64 + i64::from(index))
        .bind("tenant-001")
        .bind(action)
        .bind(format!("provider-{index:05}"))
        .bind(request_id)
        .bind(trace_id)
        .execute(&pool)
        .await
        .expect("seed audit events should succeed");
    }

    // Refresh planner statistics: without `ANALYZE` the planner still believes
    // the table is empty and picks a sequential/primary-key plan, which would
    // make these assertions test the autovacuum schedule rather than the
    // indexes.
    sqlx::query("ANALYZE dr_drive_audit_event")
        .execute(&pool)
        .await
        .expect("refresh audit event statistics");

    // The index is `(action, created_at DESC)`, so it only satisfies an
    // `ORDER BY created_at DESC`. Ordering by `id` would let the planner walk
    // the primary-key index instead — a legitimate choice, but one that would
    // leave the index under test unexercised. The three probes below therefore
    // mirror the access paths the production list/count queries actually use.
    assert_query_plan_uses_index(
        &pool,
        "EXPLAIN
         SELECT id
         FROM dr_drive_audit_event
         WHERE action = $1
         ORDER BY created_at DESC
         LIMIT $2 OFFSET $3",
        &[
            PlanBind::Text("drive.storage_provider.secret_rotated"),
            PlanBind::Int(20),
            PlanBind::Int(0),
        ],
        "ix_dr_drive_audit_event_action_created",
    )
    .await;

    assert_query_plan_uses_index(
        &pool,
        "EXPLAIN
         SELECT COUNT(1)
         FROM dr_drive_audit_event
         WHERE request_id = $1",
        &[PlanBind::Text("request-rare")],
        "ix_dr_drive_audit_event_request_created",
    )
    .await;

    assert_query_plan_uses_index(
        &pool,
        "EXPLAIN
         SELECT COUNT(1)
         FROM dr_drive_audit_event
         WHERE trace_id = $1",
        &[PlanBind::Text("trace-rare")],
        "ix_dr_drive_audit_event_trace_created",
    )
    .await;
}

/// A bind value for a query-plan probe.
///
/// PostgreSQL is strict about parameter types: `LIMIT`/`OFFSET` only accept
/// `bigint`, while the filter predicates are text. Passing everything as a
/// string — as this test originally did — fails with `42804` before the plan is
/// even produced.
enum PlanBind<'a> {
    Text(&'a str),
    Int(i64),
}

async fn assert_query_plan_uses_index(
    pool: &sqlx::PgPool,
    // sqlx 0.9 requires the SQL text to be `&'static str`; every call site
    // passes a literal EXPLAIN statement.
    sql: &'static str,
    binds: &[PlanBind<'_>],
    expected_index_name: &str,
) {
    let mut query = sqlx::query(sql);
    for bind in binds {
        query = match bind {
            PlanBind::Text(value) => query.bind(*value),
            PlanBind::Int(value) => query.bind(*value),
        };
    }
    let rows = query
        .fetch_all(pool)
        .await
        .expect("query plan should be available");
    // PostgreSQL returns the plan as text rows under a `QUERY PLAN` column (one
    // row per line), unlike SQLite's `EXPLAIN QUERY PLAN`, which projects a
    // `detail` column.
    let plan_details = rows
        .iter()
        .map(|row| row.get::<String, _>("QUERY PLAN"))
        .collect::<Vec<_>>();

    assert!(
        plan_details
            .iter()
            .any(|detail| detail.contains(expected_index_name)),
        "expected query plan to use index {expected_index_name}, got: {:?}",
        plan_details
    );
}
