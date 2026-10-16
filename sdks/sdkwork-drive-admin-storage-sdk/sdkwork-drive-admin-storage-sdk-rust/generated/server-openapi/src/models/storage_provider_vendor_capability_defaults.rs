use serde::{Deserialize, Serialize};

/// The encryption modes and storage classes a vendor accepts, in the contract's preference order ([0] is the default a fresh configuration carries). Carried on the bootstrap response so the console's editor dropdowns and the server's own capability report read one table instead of two that can drift. Absent for a credential-free kind; an empty list means the vendor offers no choice.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StorageProviderVendorCapabilityDefaults {
    /// Server-side encryption tokens this vendor accepts (AES256, KMS, aws:kms, ...), preferred first.
    #[serde(rename = "serverSideEncryptionModes")]
    pub server_side_encryption_modes: Vec<String>,

    /// Storage classes this vendor accepts (STANDARD, IA, VAULT, ...), the standard tier first.
    #[serde(rename = "storageClasses")]
    pub storage_classes: Vec<String>,
}
