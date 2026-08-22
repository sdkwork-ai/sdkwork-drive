export interface CreateDriveSandboxFileRequest {
  /** Canonical sandbox-relative logical parent path. The empty value selects the sandbox root. */
  parentPath: string;
  /** Portable single file name. Separators, dot segments, control characters, and operating-system reserved names are rejected. */
  name: string;
  /** UTF-8 text or canonical base64 content. Decoded content is limited to 4194304 bytes. */
  content: string;
  encoding: 'utf8' | 'base64';
}
