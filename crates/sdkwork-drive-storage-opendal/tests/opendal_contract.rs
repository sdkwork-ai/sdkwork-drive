use sdkwork_drive_storage_contract::{
    CreateBucketRequest, CreateMultipartUploadRequest, DriveObjectLocator, DriveObjectStore,
    DriveObjectStoreErrorKind, DriveStorageProviderKind, ListBucketsRequest,
};
use sdkwork_drive_storage_opendal::{
    OpendalS3DriveObjectStore, OpendalS3ProviderParts, OpendalS3ProviderProfile,
    OpendalS3StoreConfig,
};

#[test]
fn opendal_config_builds_explicit_cloud_provider_profiles() {
    for (kind, endpoint, expected_profile) in [
        (
            "aliyun_oss",
            "https://oss-cn-hangzhou.aliyuncs.com",
            OpendalS3ProviderProfile::AliyunOss,
        ),
        (
            "tencent_cos",
            "https://cos.ap-guangzhou.myqcloud.com",
            OpendalS3ProviderProfile::TencentCos,
        ),
        (
            "huawei_obs",
            "https://obs.cn-north-4.myhuaweicloud.com",
            OpendalS3ProviderProfile::HuaweiObs,
        ),
        (
            "volcengine_tos",
            "https://tos-cn-beijing.volces.com",
            OpendalS3ProviderProfile::VolcengineTos,
        ),
    ] {
        let config = opendal_config_parts(
            kind,
            endpoint,
            Some("cn-north-1"),
            Some("tenant-a/space-a"),
            None,
        )
        .with_security(Some("AES256"), Some("STANDARD"))
        .expect("explicit cloud provider should build");

        assert_eq!(config.provider_profile, expected_profile);
        assert!(config.strict_tls);
        assert!(
            !config.force_path_style,
            "public cloud object stores should default to virtual-host style"
        );
        assert_eq!(config.root.as_deref(), Some("tenant-a/space-a"));
        assert_eq!(config.server_side_encryption.as_deref(), Some("AES256"));
        assert_eq!(config.default_storage_class.as_deref(), Some("STANDARD"));
    }
}

#[test]
fn opendal_store_reports_object_level_capabilities_without_drive_multipart() {
    let store = OpendalS3DriveObjectStore::new(test_config())
        .expect("OpenDAL S3 store should be constructed without connecting");

    assert_eq!(
        store.provider_kind(),
        DriveStorageProviderKind::S3Compatible
    );
    let capabilities = store.capabilities();
    assert!(!capabilities.supports_multipart_upload);
    assert!(!capabilities.supports_presigned_upload_part);
    assert!(capabilities.supports_presigned_download);
    assert!(capabilities.supports_range_read);
    assert!(capabilities.supports_server_side_copy);
    assert!(!capabilities.supports_versioning);
}

#[tokio::test]
async fn opendal_store_does_not_fake_drive_multipart_or_bucket_admin() {
    let store = OpendalS3DriveObjectStore::new(test_config())
        .expect("OpenDAL S3 store should be constructed without connecting");
    let locator = DriveObjectLocator {
        bucket: "drive-bucket".to_string(),
        object_key: "objects/file.bin".to_string(),
    };

    let multipart_err = store
        .create_multipart_upload(CreateMultipartUploadRequest {
            locator,
            content_type: Some("application/octet-stream".to_string()),
            metadata: Default::default(),
            checksum_sha256_hex: None,
        })
        .await
        .expect_err("Drive multipart upload must be unsupported by the OpenDAL plugin");
    assert_eq!(multipart_err.kind, DriveObjectStoreErrorKind::NotSupported);

    let bucket_err = store
        .create_bucket(CreateBucketRequest {
            bucket: "drive-bucket".to_string(),
        })
        .await
        .expect_err("bucket admin must be handled by the full S3 adapter");
    assert_eq!(bucket_err.kind, DriveObjectStoreErrorKind::NotSupported);

    let bucket_list_err = store
        .list_buckets(ListBucketsRequest)
        .await
        .expect_err("bucket discovery must be handled by the full S3 adapter");
    assert_eq!(
        bucket_list_err.kind,
        DriveObjectStoreErrorKind::NotSupported
    );
}

#[test]
fn opendal_config_rejects_unsupported_provider_kinds_and_invalid_roots() {
    let unsupported = opendal_config_parts(
        "local_filesystem",
        "https://s3.amazonaws.com",
        Some("us-east-1"),
        None,
        Some(true),
    )
    .expect_err("local filesystem must not build as an OpenDAL S3 plugin");
    assert_eq!(unsupported.kind, DriveObjectStoreErrorKind::InvalidRequest);

    let invalid_root = opendal_config_parts(
        "s3_compatible",
        "https://s3.amazonaws.com",
        Some("us-east-1"),
        Some("../escape"),
        Some(true),
    )
    .expect_err("root must be a normalized relative object prefix");
    assert_eq!(invalid_root.kind, DriveObjectStoreErrorKind::InvalidRequest);
}

#[test]
fn opendal_config_rejects_untrimmed_endpoint_and_default_bucket() {
    let endpoint_err = opendal_config_parts(
        "s3_compatible",
        " https://s3.amazonaws.com ",
        Some("us-east-1"),
        None,
        Some(false),
    )
    .expect_err("OpenDAL config should reject untrimmed endpoint_url");
    assert_eq!(endpoint_err.kind, DriveObjectStoreErrorKind::InvalidRequest);
    assert!(endpoint_err.message.contains("endpoint_url"));

    let bucket_err = OpendalS3StoreConfig::from_provider_parts(OpendalS3ProviderParts {
        default_bucket: " drive-bucket ",
        ..default_opendal_provider_parts("s3_compatible", "https://s3.amazonaws.com")
    })
    .expect_err("OpenDAL config should reject untrimmed default_bucket");
    assert_eq!(bucket_err.kind, DriveObjectStoreErrorKind::InvalidRequest);
    assert!(bucket_err.message.contains("default_bucket"));
}

#[test]
fn opendal_config_enforces_provider_strict_tls_policy() {
    let https_config = opendal_config_parts(
        "s3_compatible",
        "https://s3.amazonaws.com",
        Some("us-east-1"),
        None,
        Some(false),
    )
    .expect("https OpenDAL config should default to strict TLS");
    assert!(https_config.strict_tls);

    let private_http_config = opendal_config_parts(
        "s3_compatible",
        "http://127.0.0.1:9000",
        Some("us-east-1"),
        None,
        Some(true),
    )
    .expect("private http OpenDAL config should default to non-strict TLS");
    assert!(!private_http_config.strict_tls);

    let strict_http = opendal_config_parts(
        "s3_compatible",
        "http://127.0.0.1:9000",
        Some("us-east-1"),
        None,
        Some(true),
    )
    .with_strict_tls(Some(true))
    .expect_err("strict TLS must reject http OpenDAL endpoints");
    assert_eq!(strict_http.kind, DriveObjectStoreErrorKind::InvalidRequest);
    assert!(strict_http
        .message
        .contains("strict_tls=true requires an https endpoint"));
}

#[test]
fn opendal_config_resolves_secret_style_credential_ref_from_env_projection() {
    with_env_lock(|| {
        std::env::set_var(
            "SDKWORK_DRIVE_STORAGE_CREDENTIAL__prod_main__ACCESS_KEY_ID",
            "opendal-secret-access",
        );
        std::env::set_var(
            "SDKWORK_DRIVE_STORAGE_CREDENTIAL__prod_main__SECRET_ACCESS_KEY",
            "opendal-secret-key",
        );

        let config = OpendalS3StoreConfig::from_provider_parts(OpendalS3ProviderParts {
            force_path_style: Some(false),
            credential_ref: Some("kms:prod/main"),
            ..default_opendal_provider_parts("s3_compatible", "https://s3.amazonaws.com")
        })
        .expect("external credential_ref should resolve through env projection");

        assert_eq!(config.access_key_id, "opendal-secret-access");
        assert_eq!(config.secret_access_key, "opendal-secret-key");
        assert_eq!(config.session_token, None);
    });
}

#[test]
fn opendal_config_rejects_unmaterialized_external_secret_refs() {
    with_env_lock(|| {
        std::env::remove_var("SDKWORK_DRIVE_STORAGE_CREDENTIAL__missing_secret__ACCESS_KEY_ID");
        std::env::remove_var("SDKWORK_DRIVE_STORAGE_CREDENTIAL__missing_secret__SECRET_ACCESS_KEY");

        let err = OpendalS3StoreConfig::from_provider_parts(OpendalS3ProviderParts {
            force_path_style: Some(false),
            credential_ref: Some("secret:missing/secret"),
            ..default_opendal_provider_parts("s3_compatible", "https://s3.amazonaws.com")
        })
        .expect_err("unmaterialized external credential_ref should be rejected");

        assert_eq!(err.kind, DriveObjectStoreErrorKind::InvalidRequest);
        assert!(err
            .message
            .contains("SDKWORK_DRIVE_STORAGE_CREDENTIAL__missing_secret"));
    });
}

trait OpendalTestPartsExt<'a> {
    fn expect(self, message: &str) -> OpendalS3StoreConfig;

    fn expect_err(self, message: &str) -> sdkwork_drive_storage_contract::DriveObjectStoreError;

    fn with_security(
        self,
        server_side_encryption: Option<&'a str>,
        default_storage_class: Option<&'a str>,
    ) -> Result<OpendalS3StoreConfig, sdkwork_drive_storage_contract::DriveObjectStoreError>;

    fn with_strict_tls(
        self,
        strict_tls_override: Option<bool>,
    ) -> Result<OpendalS3StoreConfig, sdkwork_drive_storage_contract::DriveObjectStoreError>;
}

impl<'a> OpendalTestPartsExt<'a> for OpendalS3ProviderParts<'a> {
    fn expect(self, message: &str) -> OpendalS3StoreConfig {
        OpendalS3StoreConfig::from_provider_parts(self).expect(message)
    }

    fn expect_err(self, message: &str) -> sdkwork_drive_storage_contract::DriveObjectStoreError {
        OpendalS3StoreConfig::from_provider_parts(self).expect_err(message)
    }

    fn with_security(
        mut self,
        server_side_encryption: Option<&'a str>,
        default_storage_class: Option<&'a str>,
    ) -> Result<OpendalS3StoreConfig, sdkwork_drive_storage_contract::DriveObjectStoreError> {
        self.server_side_encryption = server_side_encryption;
        self.default_storage_class = default_storage_class;
        OpendalS3StoreConfig::from_provider_parts(self)
    }

    fn with_strict_tls(
        mut self,
        strict_tls_override: Option<bool>,
    ) -> Result<OpendalS3StoreConfig, sdkwork_drive_storage_contract::DriveObjectStoreError> {
        self.strict_tls_override = strict_tls_override;
        OpendalS3StoreConfig::from_provider_parts(self)
    }
}

fn opendal_config_parts<'a>(
    provider_kind: &'a str,
    endpoint_url: &'a str,
    region: Option<&'a str>,
    root: Option<&'a str>,
    force_path_style: Option<bool>,
) -> OpendalS3ProviderParts<'a> {
    OpendalS3ProviderParts {
        region,
        root,
        force_path_style,
        ..default_opendal_provider_parts(provider_kind, endpoint_url)
    }
}

fn default_opendal_provider_parts<'a>(
    provider_kind: &'a str,
    endpoint_url: &'a str,
) -> OpendalS3ProviderParts<'a> {
    OpendalS3ProviderParts {
        provider_kind,
        endpoint_url,
        region: Some("us-east-1"),
        default_bucket: "drive-bucket",
        force_path_style: Some(true),
        credential_ref: Some("plain:access-key:secret-key"),
        root: None,
        server_side_encryption: None,
        default_storage_class: None,
        strict_tls_override: None,
    }
}

fn test_config() -> OpendalS3StoreConfig {
    opendal_config_parts(
        "s3_compatible",
        "https://s3.amazonaws.com",
        Some("us-east-1"),
        None,
        Some(true),
    )
    .expect("test config should build")
}

fn with_env_lock(action: impl FnOnce()) {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    action();
}
