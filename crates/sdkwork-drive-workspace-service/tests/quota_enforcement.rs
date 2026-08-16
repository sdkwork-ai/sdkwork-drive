use sdkwork_drive_workspace_service::application::quota_enforcement::ensure_tenant_can_allocate_bytes;
use sdkwork_drive_workspace_service::infrastructure::sql::quota_store::SqlQuotaStore;
use sdkwork_drive_workspace_service::DriveServiceError;

#[tokio::test]
async fn ensure_tenant_can_allocate_bytes_respects_optional_env_limit() {
    std::env::remove_var("SDKWORK_DRIVE_TENANT_QUOTA_MAX_BYTES");
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };
    let store = SqlQuotaStore::new(pool.clone());
    ensure_tenant_can_allocate_bytes(&store, "tenant-quota", 1_000_000)
        .await
        .expect("upload should be allowed when no quota limit is configured");

    std::env::set_var("SDKWORK_DRIVE_TENANT_QUOTA_MAX_BYTES", "100");
    let error = ensure_tenant_can_allocate_bytes(&store, "tenant-quota", 101)
        .await
        .expect_err("upload should be rejected when quota would be exceeded");
    assert!(matches!(error, DriveServiceError::Validation(_)));
    std::env::remove_var("SDKWORK_DRIVE_TENANT_QUOTA_MAX_BYTES");
}
