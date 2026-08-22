export interface UpdateDriveSandboxFileContentRequest {
  /** Canonical sandbox-relative logical file path verified against entryId. */
  logicalPath: string;
  /** UTF-8 text or canonical base64 content. Decoded content is limited to 4194304 bytes. */
  content: string;
  encoding: 'utf8' | 'base64';
}
