use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CreateStorageProviderAccountRequest {
    #[serde(rename = "displayName")]
    pub display_name: String,

    /// Service provider vendor code; the storage console derives it from the selected provider kind.
    #[serde(rename = "vendorCode")]
    pub vendor_code: String,

    /// Business-agnostic account code, unique per tenant/organization.
    #[serde(rename = "accountCode")]
    pub account_code: String,

    /// The identity shape of the credential, not the account's relationship to the tenant. The storage console stores an access-key pair and therefore always records `long_term_key`; a create request naming any other shape is rejected rather than silently overwritten. `temporary_credential` and the federated / managed families are reserved for accounts that authenticate through a broker or an identity provider.
    #[serde(rename = "accountType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_type: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,

    #[serde(rename = "externalAccountId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_account_id: Option<String>,

    #[serde(rename = "regionCode")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region_code: Option<String>,

    /// Account scope. platform = a global default published by platform operators and readable from every tenant; tenant = the application tenant's own default; user = a personal account owned by one end user. platform requires a platform operator; user defaults its owner to the caller.
    #[serde(rename = "scopeType")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_type: Option<String>,

    /// Set only for user-scoped accounts: the owning user id. Naming somebody else is rejected.
    #[serde(rename = "ownerUserId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_user_id: Option<String>,

    /// Whether this account is its scope's default for the vendor + environment, i.e. what a consumer gets when it does not name an account. Promoting a new default demotes the previous one for the same vendor + environment.
    #[serde(rename = "isDefault")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,

    #[serde(rename = "accessKeyId")]
    pub access_key_id: String,

    /// Sealed into the write-only credential row; never echoed back.
    #[serde(rename = "secretAccessKey")]
    pub secret_access_key: String,

    #[serde(rename = "sessionToken")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_token: Option<String>,
}
