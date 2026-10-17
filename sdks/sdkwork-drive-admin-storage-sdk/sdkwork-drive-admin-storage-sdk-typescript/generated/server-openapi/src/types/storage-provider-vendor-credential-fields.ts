/** How the vendor names the two halves of its credential pair, plus the environment variables and console link the operator may use. Carried on the bootstrap response so the console labels an account the way the vendor does instead of maintaining a second table that can drift. */
export interface StorageProviderVendorCredentialFields {
  /** Vendor's label for the public half of the pair (SecretId, Access Key ID, Application Key ID, ...). */
  accessKeyLabel: string;
  /** Vendor's label for the private half of the pair (SecretKey, Secret Access Key, Application Key, ...). */
  secretKeyLabel: string;
  /** Vendor-namespaced environment variable name offered as the default for the public half. */
  defaultEnvAccessKey: string;
  /** Vendor-namespaced environment variable name offered as the default for the private half. */
  defaultEnvSecretKey: string;
  /** Console deep link where the operator mints the key pair. */
  consoleUrl: string;
}
