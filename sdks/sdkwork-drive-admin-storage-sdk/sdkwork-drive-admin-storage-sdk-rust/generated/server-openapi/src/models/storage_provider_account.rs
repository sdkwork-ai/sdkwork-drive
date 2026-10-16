use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StorageProviderAccount {
    pub id: String,

    /// Account scope. platform = a global default published by platform operators and readable from every tenant; tenant = the application tenant's own default; user = a personal account owned by one end user.
    #[serde(rename = "scopeType")]
    pub scope_type: String,

    /// Set only for user-scoped accounts: the owning user id.
    #[serde(rename = "ownerUserId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_user_id: Option<String>,

    /// Service provider vendor code, for example aliyun, tencent, aws, minio.
    #[serde(rename = "vendorCode")]
    pub vendor_code: String,

    /// Business-agnostic account code, unique per tenant/organization.
    #[serde(rename = "accountCode")]
    pub account_code: String,

    #[serde(rename = "displayName")]
    pub display_name: String,

    /// The identity shape of the credential, not the account's relationship to the tenant. The storage console stores an access-key pair and therefore always records `long_term_key`; a create request naming any other shape is rejected rather than silently overwritten. `temporary_credential` and the federated / managed families are reserved for accounts that authenticate through a broker or an identity provider.
    #[serde(rename = "accountType")]
    pub account_type: String,

    pub environment: String,

    /// Upstream account identifier, for example the Aliyun UID.
    #[serde(rename = "externalAccountId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_account_id: Option<String>,

    /// What the account can serve; object_storage marks storage consumers.
    #[serde(rename = "capabilityCodes")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_codes: Option<Vec<String>>,

    #[serde(rename = "regionCode")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region_code: Option<String>,

    pub status: String,

    /// Whether an active credential row exists for this account.
    #[serde(rename = "credentialConfigured")]
    pub credential_configured: bool,

    /// How many non-deleted credential rows the account owns.
    #[serde(rename = "credentialCount")]
    pub credential_count: i64,

    pub version: i64,

    /// Whether this account is its scope's default for the vendor + environment, i.e. what a consumer gets when it does not name an account.
    #[serde(rename = "isDefault")]
    pub is_default: bool,
}
