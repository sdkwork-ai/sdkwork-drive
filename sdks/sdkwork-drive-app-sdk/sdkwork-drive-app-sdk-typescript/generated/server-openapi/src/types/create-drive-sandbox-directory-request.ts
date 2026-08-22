export interface CreateDriveSandboxDirectoryRequest {
  /** Canonical sandbox-relative logical parent path. The empty value selects the sandbox root. */
  parentPath: string;
  /** Portable single directory name. Separators, dot segments, control characters, and operating-system reserved names are rejected. */
  name: string;
}
