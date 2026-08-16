use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StorageProvider {
    pub id: String,

    #[serde(rename = "providerKind")]
    pub provider_kind: String,

    pub name: String,

    #[serde(rename = "endpointUrl")]
    pub endpoint_url: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    pub bucket: String,

    #[serde(rename = "pathStyle")]
    pub path_style: bool,

    /// Drive storage credential reference. Supported forms: plain:<accessKeyId>:<secretAccessKey>[:<sessionToken>], env:<accessKeyEnv>:<secretKeyEnv>[:<sessionTokenEnv>], secret:<ref>, kms:<ref>, or vault:<ref>. secret/kms/vault refs are materialized at runtime from SDKWORK_DRIVE_STORAGE_CREDENTIAL__<sanitized_ref>__ACCESS_KEY_ID, __SECRET_ACCESS_KEY, and optional __SESSION_TOKEN environment variables.
    #[serde(rename = "credentialRef")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_ref: Option<String>,

    #[serde(rename = "serverSideEncryptionMode")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_side_encryption_mode: Option<String>,

    #[serde(rename = "defaultStorageClass")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_storage_class: Option<String>,

    pub status: String,

    pub version: i64,

    #[serde(rename = "credentialConfigured")]
    pub credential_configured: bool,

    /// Provider-level TLS policy. HTTPS endpoints default to true, private HTTP endpoints default to false, and true requires an HTTPS endpoint.
    #[serde(rename = "strictTls")]
    pub strict_tls: bool,
}
