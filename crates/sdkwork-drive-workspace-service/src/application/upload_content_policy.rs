use crate::DriveServiceError;
use sdkwork_drive_config::{is_blocked_upload_content_type, UploadContentPolicyMode};
use sdkwork_drive_observability::metrics;

#[derive(Debug, Clone)]
pub struct UploadContentPolicyContext {
    pub tenant_id: String,
    pub upload_item_id: String,
    pub content_type: String,
    pub content_length: i64,
    pub checksum_sha256_hex: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UploadContentPolicyDecision {
    Allow,
    Block,
    Quarantine,
}

/// MIME/extension upload policy hook. This is not an antivirus integration.
pub fn evaluate_upload_content_policy(
    context: &UploadContentPolicyContext,
) -> UploadContentPolicyDecision {
    let mode = UploadContentPolicyMode::from_env();
    if mode == UploadContentPolicyMode::Disabled {
        return UploadContentPolicyDecision::Allow;
    }

    metrics::record_upload_content_policy_evaluated();
    if !is_blocked_upload_content_type(&context.content_type) {
        return UploadContentPolicyDecision::Allow;
    }

    match mode {
        UploadContentPolicyMode::Disabled => UploadContentPolicyDecision::Allow,
        UploadContentPolicyMode::Audit => {
            tracing::warn!(
                target: "sdkwork.drive",
                event = "drive.upload_content_policy.blocked_type_audit",
                tenant_id = %context.tenant_id,
                upload_item_id = %context.upload_item_id,
                content_type = %context.content_type,
                "upload content type matched blocked MIME policy in audit mode"
            );
            UploadContentPolicyDecision::Allow
        }
        UploadContentPolicyMode::Enforce => UploadContentPolicyDecision::Block,
        UploadContentPolicyMode::Quarantine => UploadContentPolicyDecision::Quarantine,
    }
}

pub async fn enforce_upload_content_policy(
    context: &UploadContentPolicyContext,
) -> Result<(), DriveServiceError> {
    match evaluate_upload_content_policy(context) {
        UploadContentPolicyDecision::Allow => Ok(()),
        UploadContentPolicyDecision::Block => Err(DriveServiceError::Validation(format!(
            "upload blocked by content policy for content type {}",
            context.content_type
        ))),
        UploadContentPolicyDecision::Quarantine => Err(DriveServiceError::Validation(format!(
            "upload quarantined by content policy for content type {}",
            context.content_type
        ))),
    }
}
