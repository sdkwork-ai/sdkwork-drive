export interface SetDefaultStorageProviderBindingRequest {
  spaceId?: string;
  spaceType?: 'personal' | 'team' | 'knowledge_base' | 'ai_generated' | 'git_repository' | 'deployment' | 'app_upload' | 'im' | 'rtc' | 'notary' | 'website';
  providerId: string;
  /** Storage binding root prefix. UTF-8 1-512 bytes, trimmed relative prefix; no leading/trailing slash, double slash, NUL, or period-only path segments. */
  storageRootPrefix?: string;
}
