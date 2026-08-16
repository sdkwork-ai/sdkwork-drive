export interface StorageProviderCapabilities {
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
