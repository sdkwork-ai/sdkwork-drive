export interface CreateShareLinkRequest {
  id: string;
  /** Optional client-supplied token. When omitted, the server generates a high-entropy token and returns it once in the create response. */
  token?: string;
  role?: 'reader' | 'commenter' | 'writer';
  expiresAtEpochMs?: string;
  downloadLimit?: string;
  /** Optional extraction code required by recipients before resolving or downloading the shared link. */
  accessCode?: string;
}
