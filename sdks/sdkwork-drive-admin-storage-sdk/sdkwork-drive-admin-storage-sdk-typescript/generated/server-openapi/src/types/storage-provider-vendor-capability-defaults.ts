/** The encryption modes and storage classes a vendor accepts, in the contract's preference order ([0] is the default a fresh configuration carries). Carried on the bootstrap response so the console's editor dropdowns and the server's own capability report read one table instead of two that can drift. Absent for a credential-free kind; an empty list means the vendor offers no choice. */
export interface StorageProviderVendorCapabilityDefaults {
  /** Server-side encryption tokens this vendor accepts (AES256, KMS, aws:kms, ...), preferred first. */
  serverSideEncryptionModes: string[];
  /** Storage classes this vendor accepts (STANDARD, IA, VAULT, ...), the standard tier first. */
  storageClasses: string[];
}
