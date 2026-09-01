export interface UpdateShareLinkRequest {
  role?: 'reader' | 'commenter' | 'writer';
  expiresAtEpochMs?: string;
  downloadLimit?: string;
}
