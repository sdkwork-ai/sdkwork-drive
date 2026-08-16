use sdkwork_drive_config::{
    allows_plain_credential_refs, is_blocked_upload_content_type, is_production_runtime_profile,
    TenantQuotaPolicy, UploadContentPolicyMode,
};

#[test]
fn runtime_policy_matrix() {
    std::env::remove_var("SDKWORK_DRIVE_RUNTIME_PROFILE");
    std::env::remove_var("SDKWORK_DRIVE_TENANT_QUOTA_MAX_BYTES");
    std::env::remove_var("SDKWORK_DRIVE_UPLOAD_CONTENT_POLICY_MODE");
    std::env::remove_var("SDKWORK_DRIVE_CONTENT_SCAN_MODE");
    std::env::remove_var("SDKWORK_DRIVE_ALLOW_PLAIN_CREDENTIAL_REFS");

    std::env::set_var("SDKWORK_DRIVE_RUNTIME_PROFILE", "production");
    assert!(is_production_runtime_profile());
    assert!(!allows_plain_credential_refs());
    assert_eq!(
        UploadContentPolicyMode::from_env(),
        UploadContentPolicyMode::Enforce
    );

    std::env::set_var("SDKWORK_DRIVE_TENANT_QUOTA_MAX_BYTES", "1048576");
    assert_eq!(TenantQuotaPolicy::from_env().max_bytes, Some(1_048_576));

    assert!(is_blocked_upload_content_type(
        "application/vnd.microsoft.portable-executable"
    ));
    assert!(!is_blocked_upload_content_type("application/pdf"));

    std::env::remove_var("SDKWORK_DRIVE_RUNTIME_PROFILE");
    std::env::remove_var("SDKWORK_DRIVE_TENANT_QUOTA_MAX_BYTES");
    std::env::remove_var("SDKWORK_DRIVE_UPLOAD_CONTENT_POLICY_MODE");
    std::env::remove_var("SDKWORK_DRIVE_CONTENT_SCAN_MODE");
}
