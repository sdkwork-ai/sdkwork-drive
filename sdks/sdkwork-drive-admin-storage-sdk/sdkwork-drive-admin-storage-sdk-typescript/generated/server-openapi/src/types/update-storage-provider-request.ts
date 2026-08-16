export interface UpdateStorageProviderRequest {
  name?: string;
  endpointUrl?: string;
  region?: string;
  bucket?: string;
  pathStyle?: boolean;
  /** Drive storage credential reference. Supported forms: plain:<accessKeyId>:<secretAccessKey>[:<sessionToken>], env:<accessKeyEnv>:<secretKeyEnv>[:<sessionTokenEnv>], secret:<ref>, kms:<ref>, or vault:<ref>. secret/kms/vault refs are materialized at runtime from SDKWORK_DRIVE_STORAGE_CREDENTIAL__<sanitized_ref>__ACCESS_KEY_ID, __SECRET_ACCESS_KEY, and optional __SESSION_TOKEN environment variables. */
  credentialRef?: string;
  serverSideEncryptionMode?: string;
  defaultStorageClass?: string;
  status?: string;
  /** Provider-level TLS policy. HTTPS endpoints default to true, private HTTP endpoints default to false, and true requires an HTTPS endpoint. */
  strictTls?: boolean;
}
