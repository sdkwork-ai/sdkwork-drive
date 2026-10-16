use serde::{Deserialize, Serialize};

/// How the vendor names the two halves of its credential pair, plus the environment variables and console link the operator may use. Carried on the bootstrap response so the console labels an account the way the vendor does instead of maintaining a second table that can drift.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StorageProviderVendorCredentialFields {
    /// Vendor's label for the public half of the pair (SecretId, Access Key ID, Application Key ID, ...).
    #[serde(rename = "accessKeyLabel")]
    pub access_key_label: String,

    /// Vendor's label for the private half of the pair (SecretKey, Secret Access Key, Application Key, ...).
    #[serde(rename = "secretKeyLabel")]
    pub secret_key_label: String,

    /// Vendor-namespaced environment variable name offered as the default for the public half.
    #[serde(rename = "defaultEnvAccessKey")]
    pub default_env_access_key: String,

    /// Vendor-namespaced environment variable name offered as the default for the private half.
    #[serde(rename = "defaultEnvSecretKey")]
    pub default_env_secret_key: String,

    /// Console deep link where the operator mints the key pair.
    #[serde(rename = "consoleUrl")]
    pub console_url: String,
}
