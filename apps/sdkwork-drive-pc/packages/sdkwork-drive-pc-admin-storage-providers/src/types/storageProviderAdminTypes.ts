export type StorageProviderKind =
  | 'local_filesystem'
  | 's3_compatible'
  | 'google_cloud_storage'
  | 'aliyun_oss'
  | 'tencent_cos'
  | 'huawei_obs'
  | 'volcengine_tos'
  | 'custom'
  | `custom:${string}`;

export type StorageProviderHealthStatus = 'unknown' | 'healthy' | 'degraded' | 'unreachable';

/** Built-in provider kind registry row (服务商). */
export interface StorageProviderKindView {
  providerKind: string;
  displayName: string;
  enabled: boolean;
  sortOrder: number;
  version: number;
  configCount: number;
}

export interface StorageProviderView {
  id: string;
  providerKind: string;
  displayName: string;
  endpointUrl: string;
  region?: string;
  bucket: string;
  pathStyle: boolean;
  credentialRef?: string;
  credentialConfigured: boolean;
  serverSideEncryptionMode?: string;
  defaultStorageClass?: string;
  status: string;
  version: number;
  strictTls: boolean;
  healthStatus?: StorageProviderHealthStatus;
  lastHealthCheckAt?: number;
  objectCount?: number;
  totalSizeBytes?: number;
}

export interface StorageProviderCapabilitiesView {
  providerId: string;
  providerKind: string;
  supportsMultipartUpload: boolean;
  supportsPresignedUploadPart: boolean;
  supportsPresignedDownload: boolean;
  supportsServerSideEncryption: boolean;
  supportsStorageClass: boolean;
  supportsCredentialRotation: boolean;
  supportedServerSideEncryptionModes: string[];
  supportedStorageClasses: string[];
}

export interface StorageProviderBindingView {
  id: string;
  tenantId: string;
  spaceId?: string;
  providerId: string;
  bindingScope: string;
  purpose: string;
  lifecycleStatus: string;
  version: number;
  storageRootPrefix?: string;
  storageProvider?: StorageProviderView;
}

export interface StorageProviderBucketView {
  providerId: string;
  bucket: string;
  exists: boolean;
}

export interface CreateStorageProviderInput {
  id: string;
  providerKind: StorageProviderKind;
  name: string;
  endpointUrl: string;
  region?: string;
  bucket: string;
  pathStyle?: boolean;
  credentialRef?: string;
  serverSideEncryptionMode?: string;
  defaultStorageClass?: string;
  status?: string;
  strictTls?: boolean;
}

export interface UpdateStorageProviderInput {
  name?: string;
  endpointUrl?: string;
  region?: string;
  bucket?: string;
  pathStyle?: boolean;
  credentialRef?: string;
  serverSideEncryptionMode?: string;
  defaultStorageClass?: string;
  status?: string;
  strictTls?: boolean;
}

export interface ListStorageProvidersInput {
  status?: string;
  pageSize?: number;
  pageToken?: string;
  signal?: AbortSignal;
}

export interface ListStorageProvidersPageResult {
  items: StorageProviderView[];
  nextPageToken?: string;
  hasMore: boolean;
}

export interface SetDefaultStorageProviderBindingInput {
  providerId: string;
  spaceId?: string;
  spaceType?: string;
  storageRootPrefix?: string;
  signal?: AbortSignal;
}

export interface StorageProviderBucketListItemView {
  bucket: string;
  configured: boolean;
  creationDate?: string;
}

export interface StorageProviderObjectView {
  key: string;
  sizeBytes: number;
  contentType?: string;
  etag?: string;
  lastModified?: string;
  isFolder: boolean;
}

export interface ListStorageProviderObjectsInput {
  prefix?: string;
  pageToken?: string;
  pageSize?: number;
  signal?: AbortSignal;
}

export interface ListStorageProviderObjectsResult {
  items: StorageProviderObjectView[];
  nextPageToken?: string;
  hasMore: boolean;
}

/** 对象内容读取结果：内容以 base64 返回（受后端 8 MiB 读取上限约束）。 */
export interface StorageProviderObjectContentView {
  providerId: string;
  bucket: string;
  objectKey: string;
  contentType?: string;
  sizeBytes: number;
  encoding: 'base64';
  content: string;
  checksumSha256: string;
}

export interface WriteStorageProviderObjectContentInput {
  content: string;
  encoding?: 'utf8' | 'base64';
  contentType?: string;
}

export interface CopyStorageProviderObjectInput {
  sourceObjectKey: string;
  destinationObjectKey: string;
}

export interface StorageProviderObjectMutationResult {
  providerId: string;
  bucket: string;
  objectKey: string;
  changed: boolean;
}

export interface StorageProviderMutationOptions {
  signal?: AbortSignal;
}
