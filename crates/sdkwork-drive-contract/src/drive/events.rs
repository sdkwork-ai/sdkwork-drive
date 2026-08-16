//! Versioned cross-service Drive event contracts.

use sdkwork_utils_rust::{hmac_sha256, hmac_sha256_base64url, secure_compare, sha256_hash};
use serde::{Deserialize, Serialize};

pub const EVENT_SPEC_VERSION: &str = "1.0";
pub const EVENT_SOURCE: &str = "sdkwork-drive";
pub const WEBHOOK_EVENT_ID_HEADER: &str = "x-sdkwork-event-id";
pub const WEBHOOK_EVENT_TIMESTAMP_HEADER: &str = "x-sdkwork-event-timestamp";
pub const WEBHOOK_EVENT_SIGNATURE_HEADER: &str = "x-sdkwork-event-signature";
pub const WEBHOOK_EVENT_RETRY_COUNT_HEADER: &str = "x-sdkwork-event-retry-count";
pub const WEBHOOK_CHANNEL_ID_HEADER: &str = "x-sdkwork-drive-channel-id";
pub const WEBHOOK_IDEMPOTENCY_KEY_HEADER: &str = "x-sdkwork-idempotency-key";
pub const WEBHOOK_SIGNATURE_VERSION: &str = "v1";
pub const WEBHOOK_SIGNING_TOKEN_MIN_LENGTH: usize = 32;
pub const WEBHOOK_SIGNING_TOKEN_MAX_LENGTH: usize = 1_024;
pub const WEBSITE_EVENT_CHANNEL_PREFIX: &str = "web:";
pub const WEBSITE_PROVIDER_EVENT_SUBSCRIPTION_ID: &str = "drive-website-events";

const WEBSITE_EVENT_CHANNEL_DERIVATION_DOMAIN: &str = "sdkwork.drive.website-event-channel.v1";
const WEBSITE_EVENT_TOKEN_DERIVATION_DOMAIN: &str = "sdkwork.drive.website-event-token.v1";

/// Derives the stable Drive channel identifier for one Web Node and WebsiteRoot.
///
/// The 60 hexadecimal digest characters plus the `web:` prefix fit the Drive channel contract's
/// 64-byte limit while retaining 240 bits of collision resistance.
pub fn derive_website_event_channel_id(node_uuid: &str, website_root_uuid: &str) -> String {
    let context =
        format!("{WEBSITE_EVENT_CHANNEL_DERIVATION_DOMAIN}\0{node_uuid}\0{website_root_uuid}");
    let digest = sha256_hash(context.as_bytes());
    format!("{WEBSITE_EVENT_CHANNEL_PREFIX}{}", &digest[..60])
}

/// Derives an independent, write-only verification token for one Node/root Drive channel.
pub fn derive_website_event_verification_token(
    channel_id: &str,
    node_derivation_secret: &[u8],
) -> String {
    let context = format!("{WEBSITE_EVENT_TOKEN_DERIVATION_DOMAIN}\0{channel_id}");
    hmac_sha256_base64url(context.as_bytes(), node_derivation_secret)
}

/// Derives the per-channel HMAC key retained by Drive from a high-entropy token.
///
/// Consumers keep the original token. Drive stores only this fixed-length digest and uses the
/// digest bytes as the webhook signing key, so the raw token is never persisted or returned.
pub fn derive_webhook_signing_key(token: &str) -> String {
    sha256_hash(token.as_bytes())
}

/// Signs `timestamp + "." + exact_body_bytes` using the versioned Drive webhook contract.
pub fn sign_webhook(timestamp: &str, exact_body: &[u8], signing_key: &[u8]) -> String {
    let payload = webhook_signing_payload(timestamp, exact_body);
    format!(
        "{WEBHOOK_SIGNATURE_VERSION}={}",
        hmac_sha256(&payload, signing_key)
    )
}

/// Verifies a versioned Drive webhook signature in constant time after structural validation.
pub fn verify_webhook_signature(
    timestamp: &str,
    exact_body: &[u8],
    signing_key: &[u8],
    signature: &str,
) -> bool {
    let expected = sign_webhook(timestamp, exact_body, signing_key);
    secure_compare(&expected, signature)
}

fn webhook_signing_payload(timestamp: &str, exact_body: &[u8]) -> Vec<u8> {
    let mut payload = Vec::with_capacity(timestamp.len() + 1 + exact_body.len());
    payload.extend_from_slice(timestamp.as_bytes());
    payload.push(b'.');
    payload.extend_from_slice(exact_body);
    payload
}

/// CloudEvents-aligned envelope used by Drive's cross-service event authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveEventEnvelope<T> {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub source: String,
    pub specversion: String,
    pub time: String,
    pub tenant_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    pub subject: String,
    pub actor_id: String,
    /// Per-Space monotonic checkpoint. Serialized as an int64 string.
    pub sequence_no: String,
    pub data: T,
}

impl<T> DriveEventEnvelope<T> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        event_type: impl Into<String>,
        time: impl Into<String>,
        tenant_id: impl Into<String>,
        organization_id: Option<String>,
        subject: impl Into<String>,
        actor_id: impl Into<String>,
        sequence_no: i64,
        data: T,
    ) -> Self {
        Self {
            id: id.into(),
            event_type: event_type.into(),
            source: EVENT_SOURCE.to_string(),
            specversion: EVENT_SPEC_VERSION.to_string(),
            time: time.into(),
            tenant_id: tenant_id.into(),
            organization_id,
            subject: subject.into(),
            actor_id: actor_id.into(),
            sequence_no: sequence_no.to_string(),
            data,
        }
    }
}

/// A Drive-authorized root subscription affected by a node mutation.
///
/// Consumers must act only on entries addressed to their registered scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveRootScopeEffect {
    pub scope_id: String,
    pub scope_kind: DriveRootScopeKind,
    pub relative_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_generation: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DriveRootScopeKind {
    WebsiteRoot,
    KnowledgebaseRaw,
}

/// Payload for `drive.node.version.committed.v1`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveNodeVersionCommittedV1Data {
    pub operation_id: String,
    pub space_id: String,
    pub node_id: String,
    pub drive_uri: String,
    pub drive_version_id: String,
    /// Logical Drive version number, serialized as an int64 string.
    pub version_no: String,
    pub space_relative_path: String,
    pub content_type: String,
    /// Content length in bytes, serialized as an int64 string.
    pub content_length: String,
    pub checksum_sha256_hex: String,
    pub root_scopes: Vec<DriveRootScopeEffect>,
}

/// Payload for `drive.node.path.changed.v1`.
///
/// Both root-scope sets are authoritative. Consumers must not reconstruct
/// membership by walking an unqualified node path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveNodePathChangedV1Data {
    pub operation_id: String,
    pub space_id: String,
    pub node_id: String,
    pub drive_uri: String,
    pub old_space_relative_path: String,
    pub new_space_relative_path: String,
    pub old_root_scopes: Vec<DriveRootScopeEffect>,
    pub new_root_scopes: Vec<DriveRootScopeEffect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DriveNodeEligibility {
    Eligible,
    Ineligible,
}

/// Payload for `drive.node.eligibility.changed.v1`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveNodeEligibilityChangedV1Data {
    pub operation_id: String,
    pub space_id: String,
    pub node_id: String,
    pub drive_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drive_version_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_no: Option<String>,
    pub space_relative_path: String,
    pub old_eligibility: DriveNodeEligibility,
    pub new_eligibility: DriveNodeEligibility,
    pub reason: String,
    pub root_scopes: Vec<DriveRootScopeEffect>,
}

/// Payload for `drive.node.deleted.v1`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveNodeDeletedV1Data {
    pub operation_id: String,
    pub space_id: String,
    pub node_id: String,
    pub drive_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drive_version_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_no: Option<String>,
    pub last_space_relative_path: String,
    pub deletion_reason: String,
    pub root_scopes: Vec<DriveRootScopeEffect>,
}

/// Payload for `drive.website_root.generation.changed.v1`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveWebsiteRootGenerationChangedV1Data {
    pub operation_id: String,
    pub space_id: String,
    pub website_root_uuid: String,
    pub previous_root_node_id: String,
    pub root_node_id: String,
    /// Previous logical root generation, serialized as an int64 string.
    pub previous_generation: String,
    /// Newly active logical root generation, serialized as an int64 string.
    pub generation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_sha256: Option<String>,
    /// File count for the newly active tree, serialized as an int64 string.
    pub file_count: String,
    /// Total bytes for the newly active tree, serialized as an int64 string.
    pub total_bytes: String,
    pub change_reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn website_event_channel_and_token_derivation_has_a_stable_vector() {
        assert_eq!(
            WEBSITE_PROVIDER_EVENT_SUBSCRIPTION_ID,
            "drive-website-events"
        );
        let channel_id =
            derive_website_event_channel_id("node-1", "11111111-1111-4111-8111-111111111111");
        assert_eq!(
            channel_id,
            "web:a56a1a8c243f49b7245ccb2ae7a23552d25717cb8e1609cbc152bb58cdb6"
        );
        assert_eq!(channel_id.len(), 64);
        assert_eq!(
            derive_website_event_verification_token(
                &channel_id,
                b"test-only-node-derivation-secret-32-bytes",
            ),
            "6OMVdPmSy7B2fksxP6E6eBhOcdW7zen27cFldxbhRwU"
        );
        assert_ne!(
            channel_id,
            derive_website_event_channel_id("node-1", "22222222-2222-4222-8222-222222222222",)
        );
    }

    #[test]
    fn webhook_signature_covers_timestamp_and_exact_body_bytes() {
        let key = derive_webhook_signing_key("6Yw1nJ37GZ8E0l9INjzQXklNSL4HE6Xe7n9m6hYS3jk");
        let body = br#"{"id":"event-1","data":{"path":"docs/index.md"}}"#;
        let signature = sign_webhook("1784592000", body, key.as_bytes());

        assert!(verify_webhook_signature(
            "1784592000",
            body,
            key.as_bytes(),
            &signature,
        ));
        assert!(!verify_webhook_signature(
            "1784592001",
            body,
            key.as_bytes(),
            &signature,
        ));
        assert!(!verify_webhook_signature(
            "1784592000",
            br#"{"id":"event-1", "data":{"path":"docs/index.md"}}"#,
            key.as_bytes(),
            &signature,
        ));
        assert!(!verify_webhook_signature(
            "1784592000",
            body,
            derive_webhook_signing_key("YkC7_QyF7V9fw6W3LMM5ssfvj1yzC6h5X3VYpP7x3CY",).as_bytes(),
            &signature,
        ));
    }

    #[test]
    fn node_version_event_uses_cloud_event_fields_and_int64_strings() {
        let envelope = DriveEventEnvelope::new(
            "event-1",
            "drive.node.version.committed.v1",
            "2026-07-21T00:00:00.000Z",
            "tenant-1",
            Some("organization-1".to_string()),
            "drive://spaces/space-1/nodes/node-1",
            "user-1",
            42,
            DriveNodeVersionCommittedV1Data {
                operation_id: "upload-1".to_string(),
                space_id: "space-1".to_string(),
                node_id: "node-1".to_string(),
                drive_uri: "drive://spaces/space-1/nodes/node-1".to_string(),
                drive_version_id: "version-1".to_string(),
                version_no: "7".to_string(),
                space_relative_path: "docs/index.md".to_string(),
                content_type: "text/markdown".to_string(),
                content_length: "1024".to_string(),
                checksum_sha256_hex: format!("sha256:{}", "a".repeat(64)),
                root_scopes: Vec::new(),
            },
        );

        let value = serde_json::to_value(envelope).expect("event should serialize");
        assert_eq!(value["type"], "drive.node.version.committed.v1");
        assert_eq!(value["specversion"], "1.0");
        assert_eq!(value["sequenceNo"], "42");
        assert_eq!(value["data"]["versionNo"], "7");
        assert_eq!(value["data"]["contentLength"], "1024");
        assert!(value["data"].get("objectKey").is_none());
    }

    #[test]
    fn path_change_event_keeps_old_and_new_root_qualified_paths() {
        let envelope = DriveEventEnvelope::new(
            "event-2",
            "drive.node.path.changed.v1",
            "2026-07-21T00:00:00.000Z",
            "tenant-1",
            None,
            "drive://spaces/space-1/nodes/node-1",
            "user-1",
            43,
            DriveNodePathChangedV1Data {
                operation_id: "request-1".to_string(),
                space_id: "space-1".to_string(),
                node_id: "node-1".to_string(),
                drive_uri: "drive://spaces/space-1/nodes/node-1".to_string(),
                old_space_relative_path: "draft/index.md".to_string(),
                new_space_relative_path: "docs/index.md".to_string(),
                old_root_scopes: vec![DriveRootScopeEffect {
                    scope_id: "scope-1".to_string(),
                    scope_kind: DriveRootScopeKind::KnowledgebaseRaw,
                    relative_path: "draft/index.md".to_string(),
                    root_generation: None,
                }],
                new_root_scopes: vec![DriveRootScopeEffect {
                    scope_id: "scope-1".to_string(),
                    scope_kind: DriveRootScopeKind::KnowledgebaseRaw,
                    relative_path: "docs/index.md".to_string(),
                    root_generation: None,
                }],
            },
        );

        let value = serde_json::to_value(envelope).expect("event should serialize");
        assert_eq!(value["type"], "drive.node.path.changed.v1");
        assert_eq!(
            value["data"]["oldRootScopes"][0]["relativePath"],
            "draft/index.md"
        );
        assert_eq!(
            value["data"]["newRootScopes"][0]["relativePath"],
            "docs/index.md"
        );
    }

    #[test]
    fn eligibility_and_delete_events_do_not_expose_storage_locators() {
        let eligibility = DriveNodeEligibilityChangedV1Data {
            operation_id: "request-2".to_string(),
            space_id: "space-1".to_string(),
            node_id: "node-1".to_string(),
            drive_uri: "drive://spaces/space-1/nodes/node-1".to_string(),
            drive_version_id: Some("version-1".to_string()),
            version_no: Some("8".to_string()),
            space_relative_path: "docs/index.md".to_string(),
            old_eligibility: DriveNodeEligibility::Eligible,
            new_eligibility: DriveNodeEligibility::Ineligible,
            reason: "NODE_TRASHED".to_string(),
            root_scopes: Vec::new(),
        };
        let deleted = DriveNodeDeletedV1Data {
            operation_id: "request-3".to_string(),
            space_id: "space-1".to_string(),
            node_id: "node-1".to_string(),
            drive_uri: "drive://spaces/space-1/nodes/node-1".to_string(),
            drive_version_id: Some("version-1".to_string()),
            version_no: Some("8".to_string()),
            last_space_relative_path: "docs/index.md".to_string(),
            deletion_reason: "PERMANENT_DELETE".to_string(),
            root_scopes: Vec::new(),
        };

        let serialized = serde_json::to_string(&(eligibility, deleted))
            .expect("lifecycle events should serialize");
        assert!(serialized.contains("INELIGIBLE"));
        assert!(serialized.contains("lastSpaceRelativePath"));
        for forbidden in ["objectKey", "bucketName", "presignedUrl", "credential"] {
            assert!(!serialized.contains(forbidden));
        }
    }

    #[test]
    fn website_root_generation_event_uses_stable_root_identity() {
        let envelope = DriveEventEnvelope::new(
            "event-generation-1",
            "drive.website_root.generation.changed.v1",
            "2026-07-23T00:00:00.000Z",
            "tenant-1",
            None,
            "drive://spaces/space-1/website_roots/root-uuid-1",
            "user-1",
            44,
            DriveWebsiteRootGenerationChangedV1Data {
                operation_id: "sync-1".to_string(),
                space_id: "space-1".to_string(),
                website_root_uuid: "root-uuid-1".to_string(),
                previous_root_node_id: "node-generation-1".to_string(),
                root_node_id: "node-generation-2".to_string(),
                previous_generation: "1".to_string(),
                generation: "2".to_string(),
                manifest_sha256: Some(format!("sha256:{}", "a".repeat(64))),
                file_count: "2".to_string(),
                total_bytes: "19".to_string(),
                change_reason: "SYNC_ACTIVATED".to_string(),
            },
        );

        let value = serde_json::to_value(envelope).expect("event should serialize");
        assert_eq!(
            value["subject"],
            "drive://spaces/space-1/website_roots/root-uuid-1"
        );
        assert_eq!(value["data"]["generation"], "2");
        assert!(value.to_string().find("objectKey").is_none());
    }
}
