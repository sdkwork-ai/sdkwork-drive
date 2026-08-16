use sdkwork_drive_config::{is_blocked_upload_content_type, UploadContentPolicyMode};
use sdkwork_drive_workspace_service::application::upload_content_policy::{
    enforce_upload_content_policy, evaluate_upload_content_policy, UploadContentPolicyContext,
    UploadContentPolicyDecision,
};
use sdkwork_drive_workspace_service::DriveServiceError;

fn blocked_context() -> UploadContentPolicyContext {
    UploadContentPolicyContext {
        tenant_id: "tenant-001".to_string(),
        upload_item_id: "upload-001".to_string(),
        content_type: "application/x-msdownload".to_string(),
        content_length: 128,
        checksum_sha256_hex:
            "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_string(),
    }
}

fn safe_context() -> UploadContentPolicyContext {
    UploadContentPolicyContext {
        tenant_id: "tenant-001".to_string(),
        upload_item_id: "upload-002".to_string(),
        content_type: "image/png".to_string(),
        content_length: 128,
        checksum_sha256_hex:
            "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_string(),
    }
}

#[tokio::test]
async fn upload_content_policy_matrix() {
    std::env::remove_var("SDKWORK_DRIVE_UPLOAD_CONTENT_POLICY_MODE");
    std::env::remove_var("SDKWORK_DRIVE_CONTENT_SCAN_MODE");
    std::env::remove_var("SDKWORK_DRIVE_RUNTIME_PROFILE");

    std::env::set_var("SDKWORK_DRIVE_UPLOAD_CONTENT_POLICY_MODE", "disabled");
    enforce_upload_content_policy(&blocked_context())
        .await
        .expect("disabled mode should not block uploads");

    std::env::set_var("SDKWORK_DRIVE_RUNTIME_PROFILE", "production");
    std::env::remove_var("SDKWORK_DRIVE_UPLOAD_CONTENT_POLICY_MODE");
    std::env::remove_var("SDKWORK_DRIVE_CONTENT_SCAN_MODE");
    assert_eq!(
        UploadContentPolicyMode::from_env(),
        UploadContentPolicyMode::Enforce
    );
    let error = enforce_upload_content_policy(&blocked_context())
        .await
        .expect_err("production profile should block executable MIME uploads");
    assert!(matches!(error, DriveServiceError::Validation(_)));

    enforce_upload_content_policy(&safe_context())
        .await
        .expect("safe uploads should pass upload content policy");

    std::env::set_var("SDKWORK_DRIVE_CONTENT_SCAN_MODE", "quarantine");
    assert_eq!(
        evaluate_upload_content_policy(&blocked_context()),
        UploadContentPolicyDecision::Quarantine
    );
    let quarantine_error = enforce_upload_content_policy(&blocked_context())
        .await
        .expect_err("quarantine mode should reject blocked MIME uploads");
    assert!(
        matches!(quarantine_error, DriveServiceError::Validation(message) if message.contains("quarantined"))
    );

    std::env::remove_var("SDKWORK_DRIVE_RUNTIME_PROFILE");
    std::env::remove_var("SDKWORK_DRIVE_UPLOAD_CONTENT_POLICY_MODE");
    std::env::remove_var("SDKWORK_DRIVE_CONTENT_SCAN_MODE");
}

#[test]
fn blocked_upload_content_type_detects_windows_executables() {
    assert!(is_blocked_upload_content_type(
        "application/vnd.microsoft.portable-executable"
    ));
    assert!(!is_blocked_upload_content_type("application/pdf"));
}
