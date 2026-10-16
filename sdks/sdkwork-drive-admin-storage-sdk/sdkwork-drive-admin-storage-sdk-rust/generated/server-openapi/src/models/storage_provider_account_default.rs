use serde::{Deserialize, Serialize};

use crate::models::{StorageProviderVendorCapabilityDefaults, StorageProviderVendorCredentialFields};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StorageProviderAccountDefault {
    /// Built-in storage provider kind this row bootstraps.
    #[serde(rename = "providerKind")]
    pub provider_kind: String,

    /// Id of the provider configuration this kind bootstraps.
    #[serde(rename = "providerId")]
    pub provider_id: String,

    /// True when this call created the provider configuration.
    #[serde(rename = "providerCreated")]
    pub provider_created: bool,

    /// Account-center vendor the provider is bound to; absent for a credential-free kind (local_filesystem).
    #[serde(rename = "vendorCode")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vendor_code: Option<String>,

    /// Account-center account the provider is bound to; absent for a credential-free kind.
    #[serde(rename = "providerAccountId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_account_id: Option<String>,

    /// Stable account code of the bootstrapped account; absent for a credential-free kind.
    #[serde(rename = "accountCode")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_code: Option<String>,

    /// True when this call created the account; false when an account already existed and was left untouched.
    #[serde(rename = "accountCreated")]
    pub account_created: bool,

    /// True when this call wrote the placeholder access-key pair. False means an account already existed, so its credential was deliberately not overwritten.
    #[serde(rename = "credentialSeeded")]
    pub credential_seeded: bool,

    #[serde(rename = "credentialFields")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_fields: Option<StorageProviderVendorCredentialFields>,

    #[serde(rename = "vendorCapabilities")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vendor_capabilities: Option<StorageProviderVendorCapabilityDefaults>,
}
