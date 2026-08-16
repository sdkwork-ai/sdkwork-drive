use sdkwork_drive_workspace_service::application::storage_provider_kind_service::{
    DriveStorageProviderKindService, SetStorageProviderKindEnabledCommand,
};
use sdkwork_drive_workspace_service::domain::storage_provider::DriveStorageProviderKind;
use sdkwork_drive_workspace_service::infrastructure::sql::storage_provider_kind_store::SqlStorageProviderKindStore;
use sdkwork_drive_workspace_service::DriveServiceError;

fn kind_service(pool: sqlx::PgPool) -> DriveStorageProviderKindService<SqlStorageProviderKindStore> {
    DriveStorageProviderKindService::new(SqlStorageProviderKindStore::new(pool))
}

#[tokio::test]
async fn initialize_kind_catalog_is_idempotent_and_seeds_builtin_kinds() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = kind_service(pool);
    let first = service
        .initialize_storage_provider_kinds()
        .await
        .expect("first initialization should seed the catalog");
    assert_eq!(first.len(), 7);
    assert!(first.iter().all(|kind| kind.enabled));
    let kinds = first
        .iter()
        .map(|kind| kind.provider_kind.as_str())
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"aliyun_oss"));
    assert!(kinds.contains(&"tencent_cos"));
    assert!(kinds.contains(&"local_filesystem"));

    // Idempotent: re-initialization keeps the same rows and never duplicates.
    let second = service
        .initialize_storage_provider_kinds()
        .await
        .expect("second initialization should be idempotent");
    assert_eq!(second.len(), 7);
    assert!(second.iter().all(|kind| kind.enabled));
}

#[tokio::test]
async fn disable_and_enable_provider_kind_roundtrip() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = kind_service(pool.clone());
    service
        .initialize_storage_provider_kinds()
        .await
        .expect("catalog should initialize");

    let disabled = service
        .set_storage_provider_kind_enabled(SetStorageProviderKindEnabledCommand {
            provider_kind: "aliyun_oss".to_string(),
            enabled: false,
        })
        .await
        .expect("kind should be disabled");
    assert!(!disabled.enabled);

    let enabled = service
        .set_storage_provider_kind_enabled(SetStorageProviderKindEnabledCommand {
            provider_kind: "aliyun_oss".to_string(),
            enabled: true,
        })
        .await
        .expect("kind should be re-enabled");
    assert!(enabled.enabled);

    // Re-initialization must not flip the operator-managed enabled flag.
    service
        .set_storage_provider_kind_enabled(SetStorageProviderKindEnabledCommand {
            provider_kind: "tencent_cos".to_string(),
            enabled: false,
        })
        .await
        .expect("kind should be disabled");
    let after_init = service
        .initialize_storage_provider_kinds()
        .await
        .expect("re-initialization should succeed");
    let tencent = after_init
        .iter()
        .find(|kind| kind.provider_kind == "tencent_cos")
        .expect("tencent_cos kind should exist");
    assert!(!tencent.enabled);
}

#[tokio::test]
async fn kind_availability_guards_builtin_kinds_and_allows_custom() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = kind_service(pool.clone());
    service
        .initialize_storage_provider_kinds()
        .await
        .expect("catalog should initialize");

    // Registered and enabled kind is available.
    service
        .ensure_storage_provider_kind_available(&DriveStorageProviderKind::LocalFilesystem)
        .await
        .expect("registered enabled kind must be available");

    // Disabled kind is not available.
    service
        .set_storage_provider_kind_enabled(SetStorageProviderKindEnabledCommand {
            provider_kind: "aliyun_oss".to_string(),
            enabled: false,
        })
        .await
        .expect("kind should be disabled");
    let conflict = service
        .ensure_storage_provider_kind_available(&DriveStorageProviderKind::AliyunOss)
        .await
        .expect_err("disabled kind must be rejected");
    assert!(matches!(conflict, DriveServiceError::Conflict(_)));

    // Custom kinds are always available.
    service
        .ensure_storage_provider_kind_available(&DriveStorageProviderKind::Custom(
            "custom:minio".to_string(),
        ))
        .await
        .expect("custom kind must always be available");
}

#[tokio::test]
async fn kind_availability_requires_registered_catalog() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    // Never initialize the catalog: built-in kinds must be reported as
    // not initialized rather than silently accepted.
    let service = kind_service(pool);
    let not_found = service
        .ensure_storage_provider_kind_available(&DriveStorageProviderKind::S3Compatible)
        .await
        .expect_err("unregistered built-in kind must be rejected");
    assert!(matches!(not_found, DriveServiceError::NotFound(_)));
}
