export interface UploaderRetentionRequest {
  mode: 'temporary' | 'long_term';
  ttlSeconds?: string;
  cleanupAction?: 'soft_delete' | 'hard_delete';
  hardDeleteAfterSeconds?: string;
}
