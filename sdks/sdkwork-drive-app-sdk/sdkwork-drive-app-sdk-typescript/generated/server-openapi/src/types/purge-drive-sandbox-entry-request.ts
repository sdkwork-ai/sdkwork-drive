export interface PurgeDriveSandboxEntryRequest {
  /** Canonical sandbox-relative logical path verified against entryId. */
  logicalPath: string;
  /** Must be true to remove a non-empty directory. Files and empty directories may use false. */
  recursive: boolean;
}
