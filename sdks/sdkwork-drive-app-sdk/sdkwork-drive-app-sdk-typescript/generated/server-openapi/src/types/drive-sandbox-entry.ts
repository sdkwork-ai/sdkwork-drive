export interface DriveSandboxEntry {
  /** Stable opaque identity derived by the server from the sandbox and canonical logical path. */
  id: string;
  sandboxId: string;
  /** Opaque parent identity when available. Clients navigate by logicalPath, not by decoding this value. */
  parentId?: string | null;
  name: string;
  kind: 'directory' | 'file';
  /** Canonical sandbox-relative logical path using '/' separators. This is never a physical provider path. */
  logicalPath: string;
  /** Opaque filesystem metadata revision for view reconciliation. */
  revision: string;
}
