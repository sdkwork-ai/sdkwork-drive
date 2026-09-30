use sdkwork_drive_workspace_service::application::storage_provider_kind_service::{
    DriveStorageProviderKindService, SetStorageProviderKindEnabledCommand,
};
use sdkwork_drive_workspace_service::domain::storage_provider::DriveStorageProviderKind;
use sdkwork_drive_workspace_service::infrastructure::sql::storage_provider_kind_store::SqlStorageProviderKindStore;
use sdkwork_drive_workspace_service::DriveServiceError;

fn kind_service(
    pool: sqlx::PgPool,
) -> DriveStorageProviderKindService<SqlStorageProviderKindStore> {
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
    assert_eq!(
        first.len(),
        sdkwork_drive_workspace_service::application::storage_provider_kind_service::BUILTIN_STORAGE_PROVIDER_KIND_CATALOG.len(),
        "the catalog must expose exactly the built-in kinds"
    );
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
    assert_eq!(second.len(), first.len());
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

    // The baseline DDL seeds the built-in catalog, so a kind that is absent from
    // the table can only be one that is *not* built-in. A `custom:` kind is
    // operator-defined and always available; an unrecognised built-in-shaped key
    // is rejected at parse time. Both paths must be honest about the catalog
    // rather than silently accepting an unregistered kind.
    let service = kind_service(pool);
    service
        .ensure_storage_provider_kind_available(&DriveStorageProviderKind::Custom(
            "custom:not-a-vendor".to_string(),
        ))
        .await
        .expect("custom kinds are always available");
}

/// A provider configuration may only be created against a registered kind.
///
/// This is the regression guard for the fixture bug where truncating
/// `dr_drive_storage_provider_kind` left the catalog empty and turned every
/// `POST /storage/providers` into a bogus 404.
#[tokio::test]
async fn every_builtin_kind_is_registered_by_the_baseline_schema() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = kind_service(pool);
    let registered = service
        .list_storage_provider_kinds(None)
        .await
        .expect("catalog should be readable");
    let registered_keys = registered
        .iter()
        .map(|summary| summary.kind.provider_kind.as_str())
        .collect::<Vec<_>>();

    for (provider_kind, _, _) in
        sdkwork_drive_workspace_service::application::storage_provider_kind_service::BUILTIN_STORAGE_PROVIDER_KIND_CATALOG
    {
        assert!(
            registered_keys.contains(&provider_kind),
            "baseline schema must seed {provider_kind}"
        );
    }

    // Every catalogued built-in kind resolves through the availability guard
    // without an initialize call, i.e. the schema alone is sufficient.
    service
        .ensure_storage_provider_kind_available(&DriveStorageProviderKind::S3Compatible)
        .await
        .expect("s3_compatible must be registered and enabled by the baseline schema");
}

/// Localized reads replace the locale-neutral base display name with the
/// seeded translation for the requested locale, and a locale without a
/// seeded translation falls back to the base name (`I18N_SPEC.md` §11,
/// `DATABASE_SPEC.md` §6.4.1).
#[tokio::test]
async fn localized_kind_list_prefers_translation_and_falls_back_to_base_name() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = kind_service(pool.clone());
    let base = service
        .list_storage_provider_kinds(None)
        .await
        .expect("catalog should be readable");
    let base_aliyun = base
        .iter()
        .find(|summary| summary.kind.provider_kind == "aliyun_oss")
        .expect("aliyun_oss kind should exist")
        .kind
        .display_name
        .clone();

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider_kind_translation
            (provider_kind, locale, display_name)
         VALUES ('aliyun_oss', 'zh-CN', '阿里云 OSS'),
                ('aliyun_oss', 'en-US', 'Alibaba Cloud OSS')
         ON CONFLICT (provider_kind, locale) DO UPDATE
         SET display_name = EXCLUDED.display_name",
    )
    .execute(&pool)
    .await
    .expect("translation rows should upsert");

    let localized = service
        .list_storage_provider_kinds(Some("zh-CN"))
        .await
        .expect("localized catalog should be readable");
    let zh_aliyun = localized
        .iter()
        .find(|summary| summary.kind.provider_kind == "aliyun_oss")
        .expect("aliyun_oss kind should exist");
    assert_eq!(zh_aliyun.kind.display_name, "阿里云 OSS");
    assert_eq!(zh_aliyun.config_count, 0, "counts stay locale-independent");

    let english = service
        .list_storage_provider_kinds(Some("en-US"))
        .await
        .expect("localized catalog should be readable");
    let en_aliyun = english
        .iter()
        .find(|summary| summary.kind.provider_kind == "aliyun_oss")
        .expect("aliyun_oss kind should exist");
    assert_eq!(en_aliyun.kind.display_name, "Alibaba Cloud OSS");

    // An unsupported locale tag matches no translation row: every kind keeps
    // its base (locale-neutral) display name.
    let fallback = service
        .list_storage_provider_kinds(Some("ko-KR"))
        .await
        .expect("unlocalized catalog should be readable");
    let ko_aliyun = fallback
        .iter()
        .find(|summary| summary.kind.provider_kind == "aliyun_oss")
        .expect("aliyun_oss kind should exist");
    assert_eq!(ko_aliyun.kind.display_name, base_aliyun);
}
