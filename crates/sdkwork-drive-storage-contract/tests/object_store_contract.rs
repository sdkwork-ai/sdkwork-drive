use sdkwork_drive_storage_contract::{
    DriveObjectStoreError, DriveStorageProviderCapabilities, DriveStorageProviderKind,
};

#[test]
fn provider_kind_includes_s3_compatible() {
    assert_eq!(
        DriveStorageProviderKind::S3Compatible.as_str(),
        "s3_compatible"
    );
}

#[test]
fn provider_kind_does_not_expose_unimplemented_azure_blob_adapter() {
    let source = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/types.rs"),
    )
    .expect("storage contract types should be readable");

    assert!(
        !source.contains("AzureBlob") && !source.contains("azure_blob"),
        "storage provider kinds must not advertise Azure Blob until a concrete adapter exists"
    );
}

#[test]
fn default_s3_capabilities_cover_multipart_and_presign() {
    let capabilities = DriveStorageProviderCapabilities::default_s3_compatible();
    assert!(capabilities.supports_multipart_upload);
    assert!(capabilities.supports_presigned_upload_part);
    assert!(capabilities.supports_presigned_download);
    assert!(capabilities.supports_range_read);
}

#[test]
fn stable_error_codes_do_not_leak_vendor_sdk_surface() {
    let error = DriveObjectStoreError::upstream("request timeout");
    assert_eq!(error.code(), "upstream_error");
    assert!(!error.to_string().to_lowercase().contains("aws_sdk"));
}
