#[tokio::test]
async fn postgres_sandbox_schema_exposes_required_tables_and_indexes() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    for table_name in [
        "dr_drive_sandbox_volume",
        "dr_drive_sandbox_grant",
        "dr_drive_sandbox_mutation_operation",
    ] {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_schema=current_schema() AND table_name=$1
            )",
        )
        .bind(table_name)
        .fetch_one(&pool)
        .await
        .expect("PostgreSQL sandbox table lookup should succeed");
        assert!(exists, "expected sandbox table exists: {table_name}");
    }

    for index_name in [
        "ix_dr_drive_sandbox_volume_tenant_status",
        "ix_dr_drive_sandbox_volume_tenant_organization_status",
        "ix_dr_drive_sandbox_grant_subject",
        "ix_dr_drive_sandbox_mutation_operation_pending",
        "ix_dr_drive_sandbox_mutation_operation_sandbox_created",
    ] {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM pg_indexes
                WHERE schemaname=current_schema() AND indexname=$1
            )",
        )
        .bind(index_name)
        .fetch_one(&pool)
        .await
        .expect("PostgreSQL sandbox index lookup should succeed");
        assert!(exists, "expected sandbox index exists: {index_name}");
    }
}
