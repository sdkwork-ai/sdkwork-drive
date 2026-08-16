use sdkwork_drive_storage_contract::{
    DriveObjectLocator, DriveObjectStore, DriveStorageProviderKind, HeadObjectRequest,
    ListObjectsRequest, PresignDownloadRequest, PresignUploadPartRequest,
};
use sdkwork_drive_storage_s3::{S3DriveObjectStore, S3ProviderProfile, S3StoreConfig};

#[test]
fn s3_config_validation_rejects_empty_bucket() {
    let config = S3StoreConfig {
        provider_kind: DriveStorageProviderKind::S3Compatible,
        provider_profile: S3ProviderProfile::Minio,
        endpoint: Some("http://127.0.0.1:9000".to_string()),
        region: "us-east-1".to_string(),
        default_bucket: String::new(),
        access_key_id: "minioadmin".to_string(),
        secret_access_key: "minioadmin".to_string(),
        session_token: None,
        force_path_style: true,
        strict_tls: false,
    };

    let err = config.validate().expect_err("bucket is required");
    assert_eq!(err.code(), "invalid_request");
}

#[test]
fn s3_config_validation_rejects_non_dns_compatible_bucket_names() {
    for bucket in [
        "ab",
        "Drive-Bucket",
        "drive_bucket",
        "drive..bucket",
        "drive-bucket-",
        "-drive-bucket",
        "192.168.5.4",
        "xn--drive-bucket",
        "drive-bucket--x-s3",
        " drive-bucket ",
    ] {
        let config = S3StoreConfig {
            provider_kind: DriveStorageProviderKind::S3Compatible,
            provider_profile: S3ProviderProfile::Minio,
            endpoint: Some("http://127.0.0.1:9000".to_string()),
            region: "us-east-1".to_string(),
            default_bucket: bucket.to_string(),
            access_key_id: "minioadmin".to_string(),
            secret_access_key: "minioadmin".to_string(),
            session_token: None,
            force_path_style: true,
            strict_tls: false,
        };

        let err = config.validate().expect_err("bucket should be rejected");
        assert_eq!(err.code(), "invalid_request");
        assert!(
            err.message.contains("default_bucket"),
            "error should name default_bucket for {bucket}: {}",
            err.message
        );
    }
}

#[test]
fn s3_config_factory_rejects_non_dns_compatible_default_bucket() {
    let err = S3StoreConfig::from_provider_parts(
        "s3_compatible",
        "http://127.0.0.1:9000",
        Some("us-east-1"),
        "Drive_Bucket",
        true,
        Some("plain:minioadmin:miniosecret"),
        None,
    )
    .expect_err("factory should reject non DNS-compatible bucket names");

    assert_eq!(err.code(), "invalid_request");
    assert!(err.message.contains("default_bucket"));
}

#[test]
fn s3_config_factory_rejects_untrimmed_endpoint_and_default_bucket() {
    let endpoint_err = S3StoreConfig::from_provider_parts(
        "s3_compatible",
        " http://127.0.0.1:9000 ",
        Some("us-east-1"),
        "drive-bucket",
        true,
        Some("plain:minioadmin:miniosecret"),
        None,
    )
    .expect_err("factory should reject untrimmed endpoint_url");
    assert_eq!(endpoint_err.code(), "invalid_request");
    assert!(endpoint_err.message.contains("endpoint_url"));

    let bucket_err = S3StoreConfig::from_provider_parts(
        "s3_compatible",
        "http://127.0.0.1:9000",
        Some("us-east-1"),
        " drive-bucket ",
        true,
        Some("plain:minioadmin:miniosecret"),
        None,
    )
    .expect_err("factory should reject untrimmed default_bucket");
    assert_eq!(bucket_err.code(), "invalid_request");
    assert!(bucket_err.message.contains("default_bucket"));
}

#[test]
fn s3_config_resolve_bucket_rejects_non_dns_compatible_requested_bucket() {
    let config = S3StoreConfig::from_provider_parts(
        "s3_compatible",
        "http://127.0.0.1:9000",
        Some("us-east-1"),
        "drive-bucket",
        true,
        Some("plain:minioadmin:miniosecret"),
        None,
    )
    .expect("valid config should build");

    assert_eq!(
        config
            .resolve_bucket(" ")
            .expect("empty request should fall back to the default bucket"),
        "drive-bucket"
    );
    let err = config
        .resolve_bucket("Drive_Bucket")
        .expect_err("requested bucket should be validated");
    assert_eq!(err.code(), "invalid_request");
    assert!(err.message.contains("bucket"));
}

#[test]
fn s3_config_factory_builds_from_plain_provider_parts() {
    let config = S3StoreConfig::from_provider_parts(
        "s3_compatible",
        "http://127.0.0.1:9000",
        Some("us-west-2"),
        "sdkwork-drive-test",
        true,
        Some("plain:minioadmin:miniosecret:session-token"),
        None,
    )
    .expect("plain credential_ref should build an s3 config");

    assert_eq!(config.provider_profile, S3ProviderProfile::Minio);
    assert_eq!(config.endpoint.as_deref(), Some("http://127.0.0.1:9000"));
    assert_eq!(config.region, "us-west-2");
    assert_eq!(config.default_bucket, "sdkwork-drive-test");
    assert_eq!(config.access_key_id, "minioadmin");
    assert_eq!(config.secret_access_key, "miniosecret");
    assert_eq!(config.session_token.as_deref(), Some("session-token"));
    assert!(config.force_path_style);
    assert!(!config.strict_tls);
}

#[test]
fn s3_config_factory_resolves_secret_style_credential_ref_from_env_projection() {
    with_env_lock(|| {
        std::env::set_var(
            "SDKWORK_DRIVE_STORAGE_CREDENTIAL__prod_main__ACCESS_KEY_ID",
            "secret-access-key",
        );
        std::env::set_var(
            "SDKWORK_DRIVE_STORAGE_CREDENTIAL__prod_main__SECRET_ACCESS_KEY",
            "secret-secret-key",
        );
        std::env::set_var(
            "SDKWORK_DRIVE_STORAGE_CREDENTIAL__prod_main__SESSION_TOKEN",
            "secret-session-token",
        );

        let config = S3StoreConfig::from_provider_parts(
            "s3_compatible",
            "https://s3.amazonaws.com",
            Some("us-east-1"),
            "drive-bucket",
            false,
            Some("secret:prod/main"),
            None,
        )
        .expect("secret credential_ref should resolve through env projection");

        assert_eq!(config.access_key_id, "secret-access-key");
        assert_eq!(config.secret_access_key, "secret-secret-key");
        assert_eq!(
            config.session_token.as_deref(),
            Some("secret-session-token")
        );
    });
}

#[test]
fn s3_config_factory_rejects_unmaterialized_external_secret_refs() {
    with_env_lock(|| {
        std::env::remove_var("SDKWORK_DRIVE_STORAGE_CREDENTIAL__missing_secret__ACCESS_KEY_ID");
        std::env::remove_var("SDKWORK_DRIVE_STORAGE_CREDENTIAL__missing_secret__SECRET_ACCESS_KEY");

        let err = S3StoreConfig::from_provider_parts(
            "s3_compatible",
            "https://s3.amazonaws.com",
            Some("us-east-1"),
            "drive-bucket",
            false,
            Some("vault:missing/secret"),
            None,
        )
        .expect_err("unmaterialized external credential_ref should be rejected");

        assert_eq!(err.code(), "invalid_request");
        assert!(err
            .message
            .contains("SDKWORK_DRIVE_STORAGE_CREDENTIAL__missing_secret"));
    });
}

#[test]
fn s3_config_factory_defaults_to_provider_region_and_https_strict_tls() {
    let config = S3StoreConfig::from_provider_parts(
        "custom:cloudflare_r2",
        "https://account.r2.cloudflarestorage.com",
        None,
        "drive-bucket",
        true,
        Some("plain:r2-access:r2-secret"),
        None,
    )
    .expect("cloudflare r2 provider parts should build an s3 config");

    assert_eq!(config.provider_profile, S3ProviderProfile::CloudflareR2);
    assert_eq!(config.region, "auto");
    assert!(config.strict_tls);
    assert_eq!(config.session_token, None);
}

#[test]
fn s3_config_factory_rejects_strict_tls_for_http_endpoint() {
    let err = S3StoreConfig::from_provider_parts(
        "custom:minio",
        "http://127.0.0.1:9000",
        Some("us-east-1"),
        "drive-bucket",
        true,
        Some("plain:minioadmin:miniosecret"),
        Some(true),
    )
    .expect_err("strict_tls=true must reject http endpoints");

    assert_eq!(err.code(), "invalid_request");
    assert!(err.message.contains("strict_tls=true requires"));
}

#[test]
fn s3_config_factory_rejects_empty_endpoint_before_credentials() {
    let err = S3StoreConfig::from_provider_parts(
        "s3_compatible",
        " ",
        Some("us-east-1"),
        "drive-bucket",
        true,
        Some("plain:minioadmin:miniosecret"),
        None,
    )
    .expect_err("endpoint_url is required");

    assert_eq!(err.code(), "invalid_request");
    assert!(err.message.contains("endpoint_url"));
}

#[tokio::test]
async fn s3_store_reports_s3_compatible_capabilities() {
    let config = S3StoreConfig {
        provider_kind: DriveStorageProviderKind::S3Compatible,
        provider_profile: S3ProviderProfile::Minio,
        endpoint: Some("http://127.0.0.1:9000".to_string()),
        region: "us-east-1".to_string(),
        default_bucket: "sdkwork-drive-test".to_string(),
        access_key_id: "minioadmin".to_string(),
        secret_access_key: "minioadmin".to_string(),
        session_token: None,
        force_path_style: true,
        strict_tls: false,
    };

    let store = S3DriveObjectStore::new(config)
        .await
        .expect("s3 store should be created");
    assert_eq!(
        store.provider_kind(),
        DriveStorageProviderKind::S3Compatible
    );

    let capabilities = store.capabilities();
    assert!(capabilities.supports_multipart_upload);
    assert!(capabilities.supports_presigned_upload_part);
    assert!(capabilities.supports_presigned_download);
    assert!(capabilities.supports_range_read);
}

#[tokio::test]
async fn s3_store_preserves_explicit_cloud_provider_kind() {
    for (provider_kind, expected_kind) in [
        ("tencent_cos", DriveStorageProviderKind::TencentCos),
        ("huawei_obs", DriveStorageProviderKind::HuaweiObs),
        ("volcengine_tos", DriveStorageProviderKind::VolcengineTos),
    ] {
        let config = S3StoreConfig::from_provider_parts(
            provider_kind,
            "https://example.com",
            Some("us-east-1"),
            "drive-bucket",
            false,
            Some("plain:access-key:secret-key"),
            None,
        )
        .expect("explicit cloud provider config should build");

        let store = S3DriveObjectStore::new(config)
            .await
            .expect("store should be constructed without connecting");

        assert_eq!(
            store.provider_kind(),
            expected_kind,
            "S3 store should preserve provider_kind for {provider_kind}"
        );
    }
}

#[tokio::test]
async fn s3_store_rejects_invalid_object_keys_before_sdk_request() {
    let store = S3DriveObjectStore::new(test_config())
        .await
        .expect("s3 store should be created");

    for object_key in [
        "",
        " object-key ",
        "/leading-slash",
        "trailing-slash/",
        "objects//double-slash",
        "objects/./content",
        "objects/../content",
        "\0",
    ] {
        let err = store
            .head_object(HeadObjectRequest {
                locator: DriveObjectLocator {
                    bucket: "drive-bucket".to_string(),
                    object_key: object_key.to_string(),
                },
            })
            .await
            .expect_err("invalid object key should fail before S3 request");
        assert_eq!(err.code(), "invalid_request");
        assert!(
            err.message.contains("object_key"),
            "error should name object_key for {object_key:?}: {}",
            err.message
        );
    }

    let err = store
        .head_object(HeadObjectRequest {
            locator: DriveObjectLocator {
                bucket: "drive-bucket".to_string(),
                object_key: "a".repeat(1025),
            },
        })
        .await
        .expect_err("too-long object key should fail before S3 request");
    assert_eq!(err.code(), "invalid_request");
}

#[tokio::test]
async fn s3_store_rejects_zero_max_keys_before_sdk_request() {
    let store = S3DriveObjectStore::new(test_config())
        .await
        .expect("s3 store should be created");

    let err = store
        .list_objects(ListObjectsRequest {
            bucket: "drive-bucket".to_string(),
            prefix: None,
            delimiter: None,
            continuation_token: None,
            max_keys: 0,
        })
        .await
        .expect_err("max_keys=0 should be rejected before S3 request");

    assert_eq!(err.code(), "invalid_request");
    assert!(err.message.contains("max_keys"));
}

#[tokio::test]
async fn s3_store_rejects_invalid_list_prefix_and_delimiter_before_sdk_request() {
    let store = S3DriveObjectStore::new(test_config())
        .await
        .expect("s3 store should be created");

    for (prefix, delimiter, field_name) in [
        (Some(" objects/".to_string()), None, "prefix"),
        (Some("/objects".to_string()), None, "prefix"),
        (Some("objects//".to_string()), None, "prefix"),
        (Some("objects/../secret".to_string()), None, "prefix"),
        (None, Some("::".to_string()), "delimiter"),
    ] {
        let err = store
            .list_objects(ListObjectsRequest {
                bucket: "drive-bucket".to_string(),
                prefix,
                delimiter,
                continuation_token: None,
                max_keys: 100,
            })
            .await
            .expect_err("invalid list options should fail before S3 request");
        assert_eq!(err.code(), "invalid_request");
        assert!(
            err.message.contains(field_name),
            "error should name {field_name}: {}",
            err.message
        );
    }
}

#[tokio::test]
async fn s3_store_rejects_zero_presign_expiry_before_sdk_request() {
    let store = S3DriveObjectStore::new(test_config())
        .await
        .expect("s3 store should be created");
    let locator = DriveObjectLocator {
        bucket: "drive-bucket".to_string(),
        object_key: "objects/zero-expiry.bin".to_string(),
    };

    let upload_err = store
        .presign_upload_part(PresignUploadPartRequest {
            locator: locator.clone(),
            upload_id: "upload-zero-expiry".to_string(),
            part_number: 1,
            expires_in_seconds: 0,
        })
        .await
        .expect_err("zero upload presign expiry should be rejected before S3 request");
    assert_eq!(upload_err.code(), "invalid_request");
    assert!(upload_err.message.contains("expires_in_seconds"));

    let download_err = store
        .presign_download(PresignDownloadRequest {
            locator,
            expires_in_seconds: 0,
        })
        .await
        .expect_err("zero download presign expiry should be rejected before S3 request");
    assert_eq!(download_err.code(), "invalid_request");
    assert!(download_err.message.contains("expires_in_seconds"));
}

#[tokio::test]
#[ignore = "requires MinIO or S3-compatible endpoint"]
async fn s3_store_supports_multipart_and_presign() {
    // Integration flow is enabled in later iteration with deployments/docker-compose.minio-test.yml.
}

#[test]
fn provider_profile_infers_from_custom_vendor_key() {
    assert_eq!(
        S3ProviderProfile::from_provider_kind("custom:cloudflare_r2", None),
        S3ProviderProfile::CloudflareR2
    );
    assert_eq!(
        S3ProviderProfile::from_provider_kind("custom:tencent_cos", None),
        S3ProviderProfile::TencentCos
    );
    assert_eq!(
        S3ProviderProfile::from_provider_kind("custom:huawei_obs", None),
        S3ProviderProfile::HuaweiObs
    );
    assert_eq!(
        S3ProviderProfile::from_provider_kind("custom:volcengine_tos", None),
        S3ProviderProfile::VolcengineTos
    );
    assert_eq!(
        S3ProviderProfile::from_provider_kind("custom:tos", None),
        S3ProviderProfile::VolcengineTos
    );
    assert_eq!(
        S3ProviderProfile::from_provider_kind("custom:aws_s3", None),
        S3ProviderProfile::AwsS3
    );
    assert_eq!(
        S3ProviderProfile::from_provider_kind("custom:minio", None),
        S3ProviderProfile::Minio
    );
}

#[test]
fn provider_profile_infers_from_endpoint_when_provider_kind_is_s3_compatible() {
    assert_eq!(
        S3ProviderProfile::from_provider_kind(
            "s3_compatible",
            Some("https://abc123.r2.cloudflarestorage.com"),
        ),
        S3ProviderProfile::CloudflareR2
    );
    assert_eq!(
        S3ProviderProfile::from_provider_kind(
            "s3_compatible",
            Some("https://oss-cn-hangzhou.aliyuncs.com")
        ),
        S3ProviderProfile::AliyunOss
    );
    assert_eq!(
        S3ProviderProfile::from_provider_kind(
            "s3_compatible",
            Some("https://cos.ap-guangzhou.myqcloud.com"),
        ),
        S3ProviderProfile::TencentCos
    );
    assert_eq!(
        S3ProviderProfile::from_provider_kind(
            "s3_compatible",
            Some("https://obs.cn-north-4.myhuaweicloud.com"),
        ),
        S3ProviderProfile::HuaweiObs
    );
    assert_eq!(
        S3ProviderProfile::from_provider_kind(
            "s3_compatible",
            Some("https://tos-cn-beijing.volces.com"),
        ),
        S3ProviderProfile::VolcengineTos
    );
    assert_eq!(
        S3ProviderProfile::from_provider_kind(
            "volcengine_tos",
            Some("https://tos-cn-shanghai.volces.com"),
        ),
        S3ProviderProfile::VolcengineTos
    );
}

#[test]
fn provider_profile_defaults_match_s3_provider_expectations() {
    assert_eq!(S3ProviderProfile::CloudflareR2.default_region(), "auto");
    assert_eq!(S3ProviderProfile::AwsS3.default_region(), "us-east-1");
    assert!(S3ProviderProfile::Minio.default_force_path_style());
    assert!(!S3ProviderProfile::AwsS3.default_force_path_style());
    assert!(!S3ProviderProfile::AliyunOss.default_force_path_style());
    assert!(!S3ProviderProfile::TencentCos.default_force_path_style());
    assert!(!S3ProviderProfile::HuaweiObs.default_force_path_style());
    assert!(!S3ProviderProfile::VolcengineTos.default_force_path_style());
}

fn test_config() -> S3StoreConfig {
    S3StoreConfig {
        provider_kind: DriveStorageProviderKind::S3Compatible,
        provider_profile: S3ProviderProfile::Minio,
        endpoint: Some("http://127.0.0.1:9000".to_string()),
        region: "us-east-1".to_string(),
        default_bucket: "drive-bucket".to_string(),
        access_key_id: "minioadmin".to_string(),
        secret_access_key: "minioadmin".to_string(),
        session_token: None,
        force_path_style: true,
        strict_tls: false,
    }
}

fn with_env_lock(action: impl FnOnce()) {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    action();
}
