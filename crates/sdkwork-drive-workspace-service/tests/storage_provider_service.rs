use sdkwork_drive_workspace_service::application::storage_provider_service::{
    CreateStorageProviderCommand, DeleteStorageProviderCommand, DriveStorageProviderService,
    GetStorageProviderCommand, ListStorageProvidersCommand, RotateStorageProviderCredentialCommand,
    SetStorageProviderStatusCommand, StorageProviderCapabilitiesCommand,
    TestStorageProviderCommand, UpdateStorageProviderCommand,
};
use sdkwork_drive_workspace_service::domain::storage_provider::DriveStorageProviderKind;
use sdkwork_drive_workspace_service::infrastructure::sql::storage_provider_store::SqlStorageProviderStore;
use sdkwork_drive_workspace_service::DriveServiceError;

#[tokio::test]
async fn create_and_list_storage_providers_with_status_filter() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveStorageProviderService::new(SqlStorageProviderStore::new(pool));

    service
        .create_storage_provider(CreateStorageProviderCommand {
            id: "provider-001".to_string(),
            provider_kind: DriveStorageProviderKind::S3Compatible,
            name: "Provider 001".to_string(),
            endpoint_url: "https://s3.example.com".to_string(),
            region: Some("us-east-1".to_string()),
            bucket: "drive-bucket".to_string(),
            path_style: Some(true),
            strict_tls: None,
            credential_ref: None,
            server_side_encryption_mode: Some("AES256".to_string()),
            default_storage_class: Some("STANDARD".to_string()),
            status: Some("active".to_string()),
            operator_id: "admin-001".to_string(),
        })
        .await
        .expect("active provider should be created");
    service
        .create_storage_provider(CreateStorageProviderCommand {
            id: "provider-002".to_string(),
            provider_kind: DriveStorageProviderKind::S3Compatible,
            name: "Provider 002".to_string(),
            endpoint_url: "https://s3-alt.example.com".to_string(),
            region: Some("us-east-1".to_string()),
            bucket: "drive-bucket-alt".to_string(),
            path_style: Some(true),
            strict_tls: None,
            credential_ref: None,
            server_side_encryption_mode: None,
            default_storage_class: Some("STANDARD_IA".to_string()),
            status: Some("disabled".to_string()),
            operator_id: "admin-001".to_string(),
        })
        .await
        .expect("disabled provider should be created");

    let all_items = service
        .list_storage_providers(ListStorageProvidersCommand {
            status: None,
            offset: 0,
            limit: 201,
        })
        .await
        .expect("list all providers should succeed");
    assert_eq!(all_items.len(), 2);

    let active_items = service
        .list_storage_providers(ListStorageProvidersCommand {
            status: Some("active".to_string()),
            offset: 0,
            limit: 201,
        })
        .await
        .expect("list active providers should succeed");
    assert_eq!(active_items.len(), 1);
    assert_eq!(active_items[0].id, "provider-001");
}

#[tokio::test]
async fn create_storage_provider_rejects_duplicate_id() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveStorageProviderService::new(SqlStorageProviderStore::new(pool));
    let command = CreateStorageProviderCommand {
        id: "provider-duplicate".to_string(),
        provider_kind: DriveStorageProviderKind::S3Compatible,
        name: "Provider Duplicate".to_string(),
        endpoint_url: "https://s3.example.com".to_string(),
        region: Some("us-east-1".to_string()),
        bucket: "drive-bucket".to_string(),
        path_style: Some(true),
        strict_tls: None,
        credential_ref: None,
        server_side_encryption_mode: None,
        default_storage_class: None,
        status: Some("active".to_string()),
        operator_id: "admin-001".to_string(),
    };

    service
        .create_storage_provider(command.clone())
        .await
        .expect("first provider should be created");

    let error = service
        .create_storage_provider(command)
        .await
        .expect_err("duplicate provider id should be rejected");

    assert_eq!(
        error,
        DriveServiceError::Conflict("storage provider already exists".to_string())
    );
}

#[tokio::test]
async fn update_test_and_delete_storage_provider_flow() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveStorageProviderService::new(SqlStorageProviderStore::new(pool));
    service
        .create_storage_provider(CreateStorageProviderCommand {
            id: "provider-001".to_string(),
            provider_kind: DriveStorageProviderKind::S3Compatible,
            name: "Provider 001".to_string(),
            endpoint_url: "https://s3.example.com".to_string(),
            region: Some("us-east-1".to_string()),
            bucket: "drive-bucket".to_string(),
            path_style: Some(true),
            strict_tls: None,
            credential_ref: None,
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: Some("active".to_string()),
            operator_id: "admin-001".to_string(),
        })
        .await
        .expect("provider should be created");

    let updated = service
        .update_storage_provider(UpdateStorageProviderCommand {
            provider_id: "provider-001".to_string(),
            name: Some("Provider 001 Updated".to_string()),
            endpoint_url: Some("https://s3-updated.example.com".to_string()),
            region: Some("us-west-2".to_string()),
            bucket: Some("drive-bucket-updated".to_string()),
            path_style: Some(false),
            strict_tls: None,
            credential_ref: None,
            server_side_encryption_mode: Some("aws:kms".to_string()),
            default_storage_class: Some("INTELLIGENT_TIERING".to_string()),
            status: Some("disabled".to_string()),
            operator_id: "admin-002".to_string(),
        })
        .await
        .expect("provider should be updated");
    assert_eq!(updated.status, "disabled");
    assert_eq!(updated.endpoint_url, "https://s3-updated.example.com");
    assert_eq!(updated.name, "Provider 001 Updated");
    assert_eq!(updated.region.as_deref(), Some("us-west-2"));
    assert!(!updated.path_style);
    assert_eq!(
        updated.server_side_encryption_mode.as_deref(),
        Some("aws:kms")
    );
    assert_eq!(
        updated.default_storage_class.as_deref(),
        Some("INTELLIGENT_TIERING")
    );

    let tested = service
        .test_storage_provider(TestStorageProviderCommand {
            provider_id: "provider-001".to_string(),
        })
        .await
        .expect("provider should be testable");
    assert!(tested.reachable);

    let deleted = service
        .delete_storage_provider(DeleteStorageProviderCommand {
            provider_id: "provider-001".to_string(),
            operator_id: "admin-003".to_string(),
        })
        .await
        .expect("provider should be deleted");
    assert!(deleted.deleted);

    let all_items = service
        .list_storage_providers(ListStorageProvidersCommand {
            status: None,
            offset: 0,
            limit: 201,
        })
        .await
        .expect("list should succeed");
    assert_eq!(all_items.len(), 1);
    assert_eq!(all_items[0].status, "deleted");

    let deleted_items = service
        .list_storage_providers(ListStorageProvidersCommand {
            status: Some("deleted".to_string()),
            offset: 0,
            limit: 201,
        })
        .await
        .expect("deleted list should succeed");
    assert_eq!(deleted_items.len(), 1);
    assert_eq!(deleted_items[0].id, "provider-001");
}

#[tokio::test]
async fn delete_storage_provider_rejects_active_provider_bindings() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveStorageProviderService::new(SqlStorageProviderStore::new(pool.clone()));
    service
        .create_storage_provider(CreateStorageProviderCommand {
            id: "provider-bound-active".to_string(),
            provider_kind: DriveStorageProviderKind::S3Compatible,
            name: "Provider Bound Active".to_string(),
            endpoint_url: "https://s3.example.com".to_string(),
            region: Some("us-east-1".to_string()),
            bucket: "drive-bucket".to_string(),
            path_style: Some(true),
            strict_tls: None,
            credential_ref: None,
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: Some("active".to_string()),
            operator_id: "admin-001".to_string(),
        })
        .await
        .expect("provider should be created");

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider_binding (
            id, tenant_id, space_id, provider_id, binding_scope, purpose,
            storage_root_prefix, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'default:tenant:tenant-bound-active',
            'tenant-bound-active', NULL,
            'provider-bound-active', 'tenant', 'primary',
            'sdkwork-drive/v1/tenants/tenant-bound-active',
            'active', 1, 'admin-001', 'admin-001'
        )",
    )
    .execute(&pool)
    .await
    .expect("active provider binding should be seeded");

    let delete_error = service
        .delete_storage_provider(DeleteStorageProviderCommand {
            provider_id: "provider-bound-active".to_string(),
            operator_id: "admin-002".to_string(),
        })
        .await
        .expect_err("provider with active bindings must not be deleted");
    assert_eq!(
        delete_error,
        DriveServiceError::Conflict(
            "storage provider has active bindings; remove or disable bindings before deletion"
                .to_string()
        )
    );

    let provider = service
        .get_storage_provider(GetStorageProviderCommand {
            provider_id: "provider-bound-active".to_string(),
        })
        .await
        .expect("provider should still be readable");
    assert_eq!(provider.status, "active");
}

#[tokio::test]
async fn storage_provider_service_rejects_deleted_status_when_active_bindings_exist() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveStorageProviderService::new(SqlStorageProviderStore::new(pool.clone()));
    service
        .create_storage_provider(CreateStorageProviderCommand {
            id: "provider-status-bound".to_string(),
            provider_kind: DriveStorageProviderKind::S3Compatible,
            name: "Provider Status Bound".to_string(),
            endpoint_url: "https://s3.example.com".to_string(),
            region: Some("us-east-1".to_string()),
            bucket: "drive-bucket".to_string(),
            path_style: Some(true),
            strict_tls: None,
            credential_ref: None,
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: Some("active".to_string()),
            operator_id: "admin-001".to_string(),
        })
        .await
        .expect("provider should be created");

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider_binding (
            id, tenant_id, space_id, provider_id, binding_scope, purpose,
            storage_root_prefix, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'default:tenant:tenant-status-bound',
            'tenant-status-bound', NULL,
            'provider-status-bound', 'tenant', 'primary',
            'sdkwork-drive/v1/tenants/tenant-status-bound',
            'active', 1, 'admin-001', 'admin-001'
        )",
    )
    .execute(&pool)
    .await
    .expect("active provider binding should be seeded");

    let update_error = service
        .update_storage_provider(UpdateStorageProviderCommand {
            provider_id: "provider-status-bound".to_string(),
            name: None,
            endpoint_url: None,
            region: None,
            bucket: None,
            path_style: None,
            strict_tls: None,
            credential_ref: None,
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: Some("deleted".to_string()),
            operator_id: "admin-002".to_string(),
        })
        .await
        .expect_err("update to deleted must reject active bindings");
    assert_eq!(
        update_error,
        DriveServiceError::Conflict(
            "storage provider has active bindings; remove or disable bindings before deletion"
                .to_string()
        )
    );

    let status_error = service
        .set_storage_provider_status(SetStorageProviderStatusCommand {
            provider_id: "provider-status-bound".to_string(),
            status: "deleted".to_string(),
            operator_id: "admin-003".to_string(),
        })
        .await
        .expect_err("status change to deleted must reject active bindings");
    assert_eq!(
        status_error,
        DriveServiceError::Conflict(
            "storage provider has active bindings; remove or disable bindings before deletion"
                .to_string()
        )
    );

    let provider = service
        .get_storage_provider(GetStorageProviderCommand {
            provider_id: "provider-status-bound".to_string(),
        })
        .await
        .expect("provider should still be readable");
    assert_eq!(provider.status, "active");
}

#[tokio::test]
async fn storage_provider_service_rejects_reactivating_deleted_provider() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveStorageProviderService::new(SqlStorageProviderStore::new(pool));
    service
        .create_storage_provider(CreateStorageProviderCommand {
            id: "provider-deleted-terminal".to_string(),
            provider_kind: DriveStorageProviderKind::S3Compatible,
            name: "Provider Deleted Terminal".to_string(),
            endpoint_url: "https://s3.example.com".to_string(),
            region: Some("us-east-1".to_string()),
            bucket: "drive-bucket".to_string(),
            path_style: Some(true),
            strict_tls: None,
            credential_ref: None,
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: Some("active".to_string()),
            operator_id: "admin-001".to_string(),
        })
        .await
        .expect("provider should be created");
    service
        .delete_storage_provider(DeleteStorageProviderCommand {
            provider_id: "provider-deleted-terminal".to_string(),
            operator_id: "admin-002".to_string(),
        })
        .await
        .expect("unbound provider should be deleted");

    let update_error = service
        .update_storage_provider(UpdateStorageProviderCommand {
            provider_id: "provider-deleted-terminal".to_string(),
            name: None,
            endpoint_url: None,
            region: None,
            bucket: None,
            path_style: None,
            strict_tls: None,
            credential_ref: None,
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: Some("active".to_string()),
            operator_id: "admin-003".to_string(),
        })
        .await
        .expect_err("update must not reactivate deleted providers");
    assert_eq!(
        update_error,
        DriveServiceError::Conflict(
            "deleted storage provider cannot be reactivated; create a new provider".to_string()
        )
    );

    let status_error = service
        .set_storage_provider_status(SetStorageProviderStatusCommand {
            provider_id: "provider-deleted-terminal".to_string(),
            status: "active".to_string(),
            operator_id: "admin-004".to_string(),
        })
        .await
        .expect_err("status change must not reactivate deleted providers");
    assert_eq!(
        status_error,
        DriveServiceError::Conflict(
            "deleted storage provider cannot be reactivated; create a new provider".to_string()
        )
    );

    let provider = service
        .get_storage_provider(GetStorageProviderCommand {
            provider_id: "provider-deleted-terminal".to_string(),
        })
        .await
        .expect("provider should still be readable");
    assert_eq!(provider.status, "deleted");
}

#[tokio::test]
async fn storage_provider_service_rejects_rotating_credentials_for_deleted_provider() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveStorageProviderService::new(SqlStorageProviderStore::new(pool));
    service
        .create_storage_provider(CreateStorageProviderCommand {
            id: "provider-deleted-credential".to_string(),
            provider_kind: DriveStorageProviderKind::S3Compatible,
            name: "Provider Deleted Credential".to_string(),
            endpoint_url: "https://s3.example.com".to_string(),
            region: Some("us-east-1".to_string()),
            bucket: "drive-bucket".to_string(),
            path_style: Some(true),
            strict_tls: None,
            credential_ref: Some("plain:access:secret".to_string()),
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: Some("active".to_string()),
            operator_id: "admin-001".to_string(),
        })
        .await
        .expect("provider should be created");
    service
        .delete_storage_provider(DeleteStorageProviderCommand {
            provider_id: "provider-deleted-credential".to_string(),
            operator_id: "admin-002".to_string(),
        })
        .await
        .expect("unbound provider should be deleted");

    let error = service
        .rotate_storage_provider_credential(RotateStorageProviderCredentialCommand {
            provider_id: "provider-deleted-credential".to_string(),
            credential_ref: "plain:new-access:new-secret".to_string(),
            operator_id: "admin-003".to_string(),
        })
        .await
        .expect_err("deleted providers must not accept credential rotation");
    assert_eq!(
        error,
        DriveServiceError::Conflict(
            "deleted storage provider cannot be modified; create a new provider".to_string()
        )
    );

    let provider = service
        .get_storage_provider(GetStorageProviderCommand {
            provider_id: "provider-deleted-credential".to_string(),
        })
        .await
        .expect("provider should still be readable");
    assert_eq!(provider.status, "deleted");
    assert_eq!(
        provider.credential_ref.as_deref(),
        Some("plain:access:secret")
    );
}

#[tokio::test]
async fn storage_provider_service_rejects_location_changes_when_active_bindings_exist() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveStorageProviderService::new(SqlStorageProviderStore::new(pool.clone()));
    service
        .create_storage_provider(CreateStorageProviderCommand {
            id: "provider-location-bound".to_string(),
            provider_kind: DriveStorageProviderKind::S3Compatible,
            name: "Provider Location Bound".to_string(),
            endpoint_url: "http://127.0.0.1:9000".to_string(),
            region: Some("us-east-1".to_string()),
            bucket: "drive-bucket".to_string(),
            path_style: Some(true),
            strict_tls: Some(false),
            credential_ref: Some("plain:access:secret".to_string()),
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: Some("active".to_string()),
            operator_id: "admin-001".to_string(),
        })
        .await
        .expect("provider should be created");

    sqlx::query(
        "INSERT INTO dr_drive_storage_provider_binding (
            id, tenant_id, space_id, provider_id, binding_scope, purpose,
            storage_root_prefix, lifecycle_status, version, created_by, updated_by
        ) VALUES (
            'default:tenant:tenant-location-bound',
            'tenant-location-bound', NULL,
            'provider-location-bound', 'tenant', 'primary',
            'sdkwork-drive/v1/tenants/tenant-location-bound',
            'active', 1, 'admin-001', 'admin-001'
        )",
    )
    .execute(&pool)
    .await
    .expect("active provider binding should be seeded");

    for (field_name, patch) in [
        (
            "endpoint_url",
            UpdateStorageProviderCommand {
                provider_id: "provider-location-bound".to_string(),
                name: None,
                endpoint_url: Some("http://127.0.0.1:9001".to_string()),
                region: None,
                bucket: None,
                path_style: None,
                strict_tls: None,
                credential_ref: None,
                server_side_encryption_mode: None,
                default_storage_class: None,
                status: None,
                operator_id: "admin-002".to_string(),
            },
        ),
        (
            "bucket",
            UpdateStorageProviderCommand {
                provider_id: "provider-location-bound".to_string(),
                name: None,
                endpoint_url: None,
                region: None,
                bucket: Some("drive-bucket-new".to_string()),
                path_style: None,
                strict_tls: None,
                credential_ref: None,
                server_side_encryption_mode: None,
                default_storage_class: None,
                status: None,
                operator_id: "admin-002".to_string(),
            },
        ),
        (
            "path_style",
            UpdateStorageProviderCommand {
                provider_id: "provider-location-bound".to_string(),
                name: None,
                endpoint_url: None,
                region: None,
                bucket: None,
                path_style: Some(false),
                strict_tls: None,
                credential_ref: None,
                server_side_encryption_mode: None,
                default_storage_class: None,
                status: None,
                operator_id: "admin-002".to_string(),
            },
        ),
        (
            "strict_tls",
            UpdateStorageProviderCommand {
                provider_id: "provider-location-bound".to_string(),
                name: None,
                endpoint_url: None,
                region: None,
                bucket: None,
                path_style: None,
                strict_tls: Some(true),
                credential_ref: None,
                server_side_encryption_mode: None,
                default_storage_class: None,
                status: None,
                operator_id: "admin-002".to_string(),
            },
        ),
    ] {
        let error = service
            .update_storage_provider(patch)
            .await
            .expect_err("storage location fields must be immutable while bindings are active");
        assert_eq!(
            error,
            DriveServiceError::Conflict(format!(
                "{field_name} cannot be changed while storage provider has active bindings"
            ))
        );
    }

    let updated = service
        .update_storage_provider(UpdateStorageProviderCommand {
            provider_id: "provider-location-bound".to_string(),
            name: Some("Provider Location Bound Renamed".to_string()),
            endpoint_url: None,
            region: None,
            bucket: None,
            path_style: None,
            strict_tls: None,
            credential_ref: Some("plain:new-access:new-secret".to_string()),
            server_side_encryption_mode: Some("AES256".to_string()),
            default_storage_class: Some("STANDARD".to_string()),
            status: None,
            operator_id: "admin-003".to_string(),
        })
        .await
        .expect("non-location provider fields should remain updateable");
    assert_eq!(updated.name, "Provider Location Bound Renamed");
    assert_eq!(
        updated.credential_ref.as_deref(),
        Some("plain:new-access:new-secret")
    );
    assert_eq!(updated.endpoint_url, "http://127.0.0.1:9000");
    assert_eq!(updated.bucket, "drive-bucket");
    assert!(updated.path_style);
    assert!(!updated.strict_tls);
}

#[tokio::test]
async fn create_storage_provider_supports_custom_provider_kind_prefix() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveStorageProviderService::new(SqlStorageProviderStore::new(pool));
    let created = service
        .create_storage_provider(CreateStorageProviderCommand {
            id: "provider-custom-001".to_string(),
            provider_kind: DriveStorageProviderKind::Custom("custom:vendor_x".to_string()),
            name: "Vendor X".to_string(),
            endpoint_url: "https://custom.example.com".to_string(),
            region: Some("cn-hangzhou".to_string()),
            bucket: "custom-bucket".to_string(),
            path_style: Some(true),
            strict_tls: None,
            credential_ref: None,
            server_side_encryption_mode: None,
            default_storage_class: Some("STANDARD".to_string()),
            status: Some("active".to_string()),
            operator_id: "admin-001".to_string(),
        })
        .await
        .expect("custom provider kind should be accepted");

    assert_eq!(created.provider_kind.as_str(), "custom:vendor_x");
    assert_eq!(created.name, "Vendor X");
    assert_eq!(created.region.as_deref(), Some("cn-hangzhou"));
    assert!(created.path_style);
}

#[tokio::test]
async fn create_storage_provider_applies_provider_default_path_style_when_not_provided() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveStorageProviderService::new(SqlStorageProviderStore::new(pool));
    let created_oss = service
        .create_storage_provider(CreateStorageProviderCommand {
            id: "provider-oss-001".to_string(),
            provider_kind: DriveStorageProviderKind::AliyunOss,
            name: "Aliyun OSS".to_string(),
            endpoint_url: "https://oss-cn-hangzhou.aliyuncs.com".to_string(),
            region: Some("cn-hangzhou".to_string()),
            bucket: "oss-bucket".to_string(),
            path_style: None,
            strict_tls: None,
            credential_ref: None,
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: Some("active".to_string()),
            operator_id: "admin-001".to_string(),
        })
        .await
        .expect("oss provider should be created");
    assert!(!created_oss.path_style);

    let created_minio = service
        .create_storage_provider(CreateStorageProviderCommand {
            id: "provider-minio-001".to_string(),
            provider_kind: DriveStorageProviderKind::S3Compatible,
            name: "MinIO".to_string(),
            endpoint_url: "http://127.0.0.1:9000".to_string(),
            region: Some("us-east-1".to_string()),
            bucket: "minio-bucket".to_string(),
            path_style: None,
            strict_tls: None,
            credential_ref: None,
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: Some("active".to_string()),
            operator_id: "admin-001".to_string(),
        })
        .await
        .expect("s3 compatible provider should be created");
    assert!(created_minio.path_style);
}

#[tokio::test]
async fn storage_provider_service_persists_strict_tls_and_defaults_by_endpoint_scheme() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveStorageProviderService::new(SqlStorageProviderStore::new(pool));
    let https_provider = service
        .create_storage_provider(CreateStorageProviderCommand {
            id: "provider-https-strict-default".to_string(),
            provider_kind: DriveStorageProviderKind::S3Compatible,
            name: "HTTPS Strict Default".to_string(),
            endpoint_url: "https://s3.example.com".to_string(),
            region: Some("us-east-1".to_string()),
            bucket: "drive-bucket".to_string(),
            path_style: None,
            strict_tls: None,
            credential_ref: None,
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: Some("active".to_string()),
            operator_id: "admin-001".to_string(),
        })
        .await
        .expect("https provider should default strict_tls to true");
    assert!(https_provider.strict_tls);

    let http_provider = service
        .create_storage_provider(CreateStorageProviderCommand {
            id: "provider-http-strict-default".to_string(),
            provider_kind: DriveStorageProviderKind::S3Compatible,
            name: "HTTP Strict Default".to_string(),
            endpoint_url: "http://127.0.0.1:9000".to_string(),
            region: Some("us-east-1".to_string()),
            bucket: "minio-bucket".to_string(),
            path_style: None,
            strict_tls: None,
            credential_ref: None,
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: Some("active".to_string()),
            operator_id: "admin-001".to_string(),
        })
        .await
        .expect("http provider should default strict_tls to false");
    assert!(!http_provider.strict_tls);

    let updated = service
        .update_storage_provider(UpdateStorageProviderCommand {
            provider_id: "provider-http-strict-default".to_string(),
            name: None,
            endpoint_url: Some("https://minio.example.com".to_string()),
            region: None,
            bucket: None,
            path_style: None,
            strict_tls: Some(true),
            credential_ref: None,
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: None,
            operator_id: "admin-002".to_string(),
        })
        .await
        .expect("provider strict_tls should be updateable");
    assert!(updated.strict_tls);
    assert_eq!(updated.endpoint_url, "https://minio.example.com");
}

#[tokio::test]
async fn storage_provider_service_rejects_strict_tls_true_for_http_endpoint() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveStorageProviderService::new(SqlStorageProviderStore::new(pool));
    let create_error = service
        .create_storage_provider(CreateStorageProviderCommand {
            id: "provider-http-strict-invalid".to_string(),
            provider_kind: DriveStorageProviderKind::S3Compatible,
            name: "HTTP Strict Invalid".to_string(),
            endpoint_url: "http://127.0.0.1:9000".to_string(),
            region: Some("us-east-1".to_string()),
            bucket: "minio-bucket".to_string(),
            path_style: None,
            strict_tls: Some(true),
            credential_ref: None,
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: Some("active".to_string()),
            operator_id: "admin-001".to_string(),
        })
        .await
        .expect_err("strict_tls=true must reject http endpoints");
    assert_eq!(
        create_error,
        DriveServiceError::Validation("strict_tls=true requires an https endpoint".to_string())
    );

    service
        .create_storage_provider(CreateStorageProviderCommand {
            id: "provider-http-strict-update".to_string(),
            provider_kind: DriveStorageProviderKind::S3Compatible,
            name: "HTTP Strict Update".to_string(),
            endpoint_url: "https://s3.example.com".to_string(),
            region: Some("us-east-1".to_string()),
            bucket: "drive-bucket".to_string(),
            path_style: None,
            strict_tls: Some(true),
            credential_ref: None,
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: Some("active".to_string()),
            operator_id: "admin-001".to_string(),
        })
        .await
        .expect("https strict provider should be created");

    let update_error = service
        .update_storage_provider(UpdateStorageProviderCommand {
            provider_id: "provider-http-strict-update".to_string(),
            name: None,
            endpoint_url: Some("http://127.0.0.1:9000".to_string()),
            region: None,
            bucket: None,
            path_style: None,
            strict_tls: None,
            credential_ref: None,
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: None,
            operator_id: "admin-002".to_string(),
        })
        .await
        .expect_err("existing strict_tls=true must reject update to http endpoint");
    assert_eq!(
        update_error,
        DriveServiceError::Validation("strict_tls=true requires an https endpoint".to_string())
    );
}

#[tokio::test]
async fn create_storage_provider_supports_explicit_s3_cloud_provider_kinds() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveStorageProviderService::new(SqlStorageProviderStore::new(pool));
    for (id, provider_kind, endpoint_url, region) in [
        (
            "provider-tencent-cos",
            DriveStorageProviderKind::TencentCos,
            "https://cos.ap-guangzhou.myqcloud.com",
            "ap-guangzhou",
        ),
        (
            "provider-huawei-obs",
            DriveStorageProviderKind::HuaweiObs,
            "https://obs.cn-north-4.myhuaweicloud.com",
            "cn-north-4",
        ),
        (
            "provider-volcengine-tos",
            DriveStorageProviderKind::VolcengineTos,
            "https://tos-cn-beijing.volces.com",
            "cn-beijing",
        ),
    ] {
        let created = service
            .create_storage_provider(CreateStorageProviderCommand {
                id: id.to_string(),
                provider_kind: provider_kind.clone(),
                name: format!("Provider {id}"),
                endpoint_url: endpoint_url.to_string(),
                region: Some(region.to_string()),
                bucket: format!("{id}-bucket"),
                path_style: None,
                strict_tls: None,
                credential_ref: Some("plain:access-key:secret-key".to_string()),
                server_side_encryption_mode: Some("AES256".to_string()),
                default_storage_class: Some("STANDARD".to_string()),
                status: Some("active".to_string()),
                operator_id: "admin-001".to_string(),
            })
            .await
            .expect("explicit S3 cloud provider kind should be created");

        assert_eq!(created.provider_kind, provider_kind);
        assert!(
            !created.path_style,
            "public cloud object stores should default to virtual-host style addressing"
        );

        let capabilities = service
            .get_storage_provider_capabilities(StorageProviderCapabilitiesCommand {
                provider_id: id.to_string(),
            })
            .await
            .expect("explicit cloud provider capabilities should be returned");
        assert!(capabilities.supports_multipart_upload);
        assert!(capabilities.supports_presigned_upload_part);
        assert!(capabilities.supports_presigned_download);
        assert!(capabilities.supports_storage_class);
        assert!(capabilities.supports_credential_rotation);
    }
}

#[tokio::test]
async fn storage_provider_service_rejects_invalid_endpoint_and_bucket_before_database_write() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveStorageProviderService::new(SqlStorageProviderStore::new(pool.clone()));
    let invalid_endpoint = service
        .create_storage_provider(CreateStorageProviderCommand {
            id: "provider-invalid-endpoint".to_string(),
            provider_kind: DriveStorageProviderKind::S3Compatible,
            name: "Invalid Endpoint".to_string(),
            endpoint_url: "ftp://storage.example.com".to_string(),
            region: Some("us-east-1".to_string()),
            bucket: "drive-bucket".to_string(),
            path_style: Some(true),
            strict_tls: None,
            credential_ref: None,
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: Some("active".to_string()),
            operator_id: "admin-001".to_string(),
        })
        .await
        .expect_err("non-http storage endpoint should be rejected");
    assert!(
        matches!(
            invalid_endpoint,
            DriveServiceError::Validation(message) if message.contains("endpoint_url")
        ),
        "invalid endpoint should return validation error"
    );

    let invalid_bucket = service
        .create_storage_provider(CreateStorageProviderCommand {
            id: "provider-invalid-bucket".to_string(),
            provider_kind: DriveStorageProviderKind::S3Compatible,
            name: "Invalid Bucket".to_string(),
            endpoint_url: "https://s3.example.com".to_string(),
            region: Some("us-east-1".to_string()),
            bucket: "drive bucket".to_string(),
            path_style: Some(true),
            strict_tls: None,
            credential_ref: None,
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: Some("active".to_string()),
            operator_id: "admin-001".to_string(),
        })
        .await
        .expect_err("bucket names with spaces should be rejected");
    assert!(
        matches!(
            invalid_bucket,
            DriveServiceError::Validation(message) if message.contains("bucket")
        ),
        "invalid bucket should return validation error"
    );

    let stored_count: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM dr_drive_storage_provider")
        .fetch_one(&pool)
        .await
        .expect("storage provider count should be readable");
    assert_eq!(stored_count, 0);
}

#[tokio::test]
async fn storage_provider_service_rejects_non_dns_object_store_bucket_names() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveStorageProviderService::new(SqlStorageProviderStore::new(pool.clone()));

    for (index, bucket) in [
        "ab",
        "Drive-Bucket",
        "drive_bucket",
        "drive..bucket",
        "drive-bucket-",
        "-drive-bucket",
        "192.168.5.4",
    ]
    .into_iter()
    .enumerate()
    {
        let err = service
            .create_storage_provider(CreateStorageProviderCommand {
                id: format!("provider-invalid-dns-bucket-{index}"),
                provider_kind: DriveStorageProviderKind::S3Compatible,
                name: format!("Invalid DNS Bucket {index}"),
                endpoint_url: "https://s3.example.com".to_string(),
                region: Some("us-east-1".to_string()),
                bucket: bucket.to_string(),
                path_style: Some(true),
                strict_tls: None,
                credential_ref: None,
                server_side_encryption_mode: None,
                default_storage_class: None,
                status: Some("active".to_string()),
                operator_id: "admin-001".to_string(),
            })
            .await
            .expect_err("non DNS-compatible object-store bucket should be rejected");
        assert!(
            matches!(
                err,
                DriveServiceError::Validation(message) if message.contains("bucket")
            ),
            "invalid bucket should return validation error for {bucket}"
        );
    }

    service
        .create_storage_provider(CreateStorageProviderCommand {
            id: "provider-local-path-bucket".to_string(),
            provider_kind: DriveStorageProviderKind::LocalFilesystem,
            name: "Local Path Bucket".to_string(),
            endpoint_url: "file:///tmp/sdkwork-drive".to_string(),
            region: None,
            bucket: "Drive_Bucket".to_string(),
            path_style: Some(true),
            strict_tls: None,
            credential_ref: None,
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: Some("active".to_string()),
            operator_id: "admin-001".to_string(),
        })
        .await
        .expect("local filesystem bucket should keep relative directory naming");

    let stored_count: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM dr_drive_storage_provider")
        .fetch_one(&pool)
        .await
        .expect("storage provider count should be readable");
    assert_eq!(stored_count, 1);
}

#[tokio::test]
async fn storage_provider_service_returns_detail_capabilities_status_and_rotates_credentials() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveStorageProviderService::new(SqlStorageProviderStore::new(pool));
    service
        .create_storage_provider(CreateStorageProviderCommand {
            id: "provider-s3-ops".to_string(),
            provider_kind: DriveStorageProviderKind::S3Compatible,
            name: "Provider S3 Ops".to_string(),
            endpoint_url: "https://s3.example.com".to_string(),
            region: Some("us-east-1".to_string()),
            bucket: "drive-bucket".to_string(),
            path_style: Some(true),
            strict_tls: None,
            credential_ref: Some("plain:access:secret".to_string()),
            server_side_encryption_mode: Some("AES256".to_string()),
            default_storage_class: Some("STANDARD".to_string()),
            status: Some("active".to_string()),
            operator_id: "admin-001".to_string(),
        })
        .await
        .expect("provider should be created");

    let found = service
        .get_storage_provider(GetStorageProviderCommand {
            provider_id: "provider-s3-ops".to_string(),
        })
        .await
        .expect("provider should be returned");
    assert_eq!(found.id, "provider-s3-ops");

    let capabilities = service
        .get_storage_provider_capabilities(StorageProviderCapabilitiesCommand {
            provider_id: "provider-s3-ops".to_string(),
        })
        .await
        .expect("provider capabilities should be returned");
    assert!(capabilities.supports_multipart_upload);
    assert!(capabilities.supports_presigned_upload_part);
    assert!(capabilities.supports_presigned_download);
    assert!(capabilities.supports_credential_rotation);
    assert!(capabilities
        .supported_server_side_encryption_modes
        .contains(&"aws:kms".to_string()));
    assert!(capabilities
        .supported_storage_classes
        .contains(&"INTELLIGENT_TIERING".to_string()));

    let disabled = service
        .set_storage_provider_status(SetStorageProviderStatusCommand {
            provider_id: "provider-s3-ops".to_string(),
            status: "disabled".to_string(),
            operator_id: "admin-002".to_string(),
        })
        .await
        .expect("provider should be disabled");
    assert_eq!(disabled.status, "disabled");

    let active = service
        .set_storage_provider_status(SetStorageProviderStatusCommand {
            provider_id: "provider-s3-ops".to_string(),
            status: "active".to_string(),
            operator_id: "admin-003".to_string(),
        })
        .await
        .expect("provider should be activated");
    assert_eq!(active.status, "active");

    let rotated = service
        .rotate_storage_provider_credential(RotateStorageProviderCredentialCommand {
            provider_id: "provider-s3-ops".to_string(),
            credential_ref: "env:DRIVE_ACCESS_KEY:DRIVE_SECRET_KEY".to_string(),
            operator_id: "admin-004".to_string(),
        })
        .await
        .expect("provider credentials should be rotated");
    assert_eq!(
        rotated.credential_ref.as_deref(),
        Some("env:DRIVE_ACCESS_KEY:DRIVE_SECRET_KEY")
    );
}

#[tokio::test]
async fn storage_provider_service_rejects_invalid_status_before_database_constraints() {
    let Some((pool, _database_guard)) = sdkwork_drive_test_support::postgres_test_database().await
    else {
        return;
    };

    let service = DriveStorageProviderService::new(SqlStorageProviderStore::new(pool));
    let create_error = service
        .create_storage_provider(CreateStorageProviderCommand {
            id: "provider-invalid-status".to_string(),
            provider_kind: DriveStorageProviderKind::S3Compatible,
            name: "Invalid Status".to_string(),
            endpoint_url: "https://s3.example.com".to_string(),
            region: Some("us-east-1".to_string()),
            bucket: "drive-bucket".to_string(),
            path_style: Some(true),
            strict_tls: None,
            credential_ref: None,
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: Some("paused".to_string()),
            operator_id: "admin-001".to_string(),
        })
        .await
        .expect_err("invalid provider status should be rejected before insert");
    assert_eq!(
        create_error,
        DriveServiceError::Validation(
            "status is invalid; allowed: active, disabled, deleted".to_string()
        )
    );

    service
        .create_storage_provider(CreateStorageProviderCommand {
            id: "provider-valid-status".to_string(),
            provider_kind: DriveStorageProviderKind::S3Compatible,
            name: "Valid Status".to_string(),
            endpoint_url: "https://s3.example.com".to_string(),
            region: Some("us-east-1".to_string()),
            bucket: "drive-bucket".to_string(),
            path_style: Some(true),
            strict_tls: None,
            credential_ref: None,
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: Some("active".to_string()),
            operator_id: "admin-001".to_string(),
        })
        .await
        .expect("valid provider should be created");

    let update_error = service
        .update_storage_provider(UpdateStorageProviderCommand {
            provider_id: "provider-valid-status".to_string(),
            name: None,
            endpoint_url: None,
            region: None,
            bucket: None,
            path_style: None,
            strict_tls: None,
            credential_ref: None,
            server_side_encryption_mode: None,
            default_storage_class: None,
            status: Some("paused".to_string()),
            operator_id: "admin-002".to_string(),
        })
        .await
        .expect_err("invalid provider status should be rejected before update");
    assert_eq!(
        update_error,
        DriveServiceError::Validation(
            "status is invalid; allowed: active, disabled, deleted".to_string()
        )
    );
}
