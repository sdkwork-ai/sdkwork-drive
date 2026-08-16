export interface StorageProvider {
  id: string;
  providerKind: 'local_filesystem' | 's3_compatible' | 'google_cloud_storage' | 'aliyun_oss' | 'tencent_cos' | 'huawei_obs' | 'volcengine_tos';
  name: string;
  endpointUrl: string;
  region?: string;
  bucket: string;
  pathStyle: boolean;
  /** Drive storage credential reference. Supported forms: plain:<accessKeyId>:<secretAccessKey>[:<sessionToken>], env:<accessKeyEnv>:<secretKeyEnv>[:<sessionTokenEnv>], secret:<ref>, kms:<ref>, or vault:<ref>. secret/kms/vault refs are materialized at runtime from SDKWORK_DRIVE_STORAGE_CREDENTIAL__<sanitized_ref>__ACCESS_KEY_ID, __SECRET_ACCESS_KEY, and optional __SESSION_TOKEN environment variables. */
  credentialRef?: string;
  serverSideEncryptionMode?: string;
  defaultStorageClass?: string;
  status: string;
  version: string;
  credentialConfigured: boolean;
  /** Provider-level TLS policy. HTTPS endpoints default to true, private HTTP endpoints default to false, and true requires an HTTPS endpoint. */
  strictTls: boolean;
}
