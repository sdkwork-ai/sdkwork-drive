/**
 * File-browser command registry.
 *
 * A single mutable slot table let the file browser publish its cut / paste /
 * refresh handlers to the desktop host command dispatcher without threading
 * callbacks through the whole component tree.
 *
 * The registry is module-scoped on purpose: only one file browser is mounted at
 * a time, and the browser clears its slots on unmount so a stale closure can
 * never be invoked by a late host event.
 */

export interface DriveFileBrowserCommands {
  cut: (() => void) | null;
  paste: (() => void) | null;
  refresh: (() => void) | null;
}

export const driveFileBrowserCommandRegistry: DriveFileBrowserCommands = {
  cut: null,
  paste: null,
  refresh: null,
};

/** Runs a registered file-browser command; returns false when unowned. */
export function runDriveFileBrowserCommand(
  command: keyof DriveFileBrowserCommands,
): boolean {
  const handler = driveFileBrowserCommandRegistry[command];
  if (!handler) {
    return false;
  }
  handler();
  return true;
}
