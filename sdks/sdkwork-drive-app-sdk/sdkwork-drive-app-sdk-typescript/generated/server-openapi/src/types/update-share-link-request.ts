export interface UpdateShareLinkRequest {
  role?: 'reader' | 'commenter' | 'writer';
  expiresAtEpochMs?: string | null;
  downloadLimit?: string | null;
}
