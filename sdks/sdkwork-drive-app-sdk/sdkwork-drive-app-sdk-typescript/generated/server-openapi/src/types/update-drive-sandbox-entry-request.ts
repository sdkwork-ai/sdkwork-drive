export interface UpdateDriveSandboxEntryRequest {
  /** Current canonical sandbox-relative logical path verified against entryId. */
  logicalPath: string;
  /** Canonical sandbox-relative destination directory. */
  destinationParentPath: string;
  /** Portable destination name used for move or rename. */
  destinationName: string;
}
