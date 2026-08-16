use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StorageProviderCapabilities {
    #[serde(rename = "providerId")]
    pub provider_id: String,

    #[serde(rename = "providerKind")]
    pub provider_kind: String,

    #[serde(rename = "supportsMultipartUpload")]
    pub supports_multipart_upload: bool,

    #[serde(rename = "supportsPresignedUploadPart")]
    pub supports_presigned_upload_part: bool,

    #[serde(rename = "supportsPresignedDownload")]
    pub supports_presigned_download: bool,

    #[serde(rename = "supportsServerSideEncryption")]
    pub supports_server_side_encryption: bool,

    #[serde(rename = "supportsStorageClass")]
    pub supports_storage_class: bool,

    #[serde(rename = "supportsCredentialRotation")]
    pub supports_credential_rotation: bool,

    #[serde(rename = "supportedServerSideEncryptionModes")]
    pub supported_server_side_encryption_modes: Vec<String>,

    #[serde(rename = "supportedStorageClasses")]
    pub supported_storage_classes: Vec<String>,
}
