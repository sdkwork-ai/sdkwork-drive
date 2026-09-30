/**
 * Drive cut-clipboard model.
 *
 * Drive nodes are cut through the SDK (`nodes.move`); this module owns the
 * renderer-side "pending cut" state so a cut followed by a paste in another
 * folder resolves to one move operation.
 *
 * The model is intentionally separate from the OS clipboard: a cut may target
 * files owned by Drive that have no local path at all. When the selection also
 * has local paths, the host adapter additionally writes the OS cut marker so an
 * external Explorer / Finder paste behaves as a move.
 */

import type { DriveFile } from 'sdkwork-drive-pc-types';

export interface DriveCutSelection {
  /** Source section the files were cut from. */
  section: string;
  /** Parent folder the files were cut from. */
  parentId: string | null;
  /** The cut nodes. */
  files: DriveFile[];
}

let pendingCut: DriveCutSelection | null = null;
const listeners = new Set<(selection: DriveCutSelection | null) => void>();

function notify(): void {
  for (const listener of listeners) {
    listener(pendingCut);
  }
}

/** Reads the current pending cut, or null when nothing is cut. */
export function getDriveCutSelection(): DriveCutSelection | null {
  return pendingCut;
}

/** Records a pending cut and notifies subscribers. */
export function setDriveCutSelection(selection: DriveCutSelection | null): void {
  pendingCut = selection;
  notify();
}

/** Clears the pending cut (after a successful move, or on Escape). */
export function clearDriveCutSelection(): void {
  if (pendingCut === null) {
    return;
  }
  pendingCut = null;
  notify();
}

/** Subscribes to pending-cut changes; returns an unsubscribe function. */
export function subscribeDriveCutSelection(
  listener: (selection: DriveCutSelection | null) => void,
): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

/** True when the pending cut targets the given folder in the given section. */
export function isCutIntoSameFolder(
  selection: DriveCutSelection,
  section: string,
  parentId: string | null,
): boolean {
  return selection.section === section && selection.parentId === parentId;
}

/** Local absolute path for a Drive node, when the node carries one. */
export function resolveCutLocalPath(file: DriveFile): string | null {
  const candidate = (file as DriveFile & { localPath?: string; path?: string }).localPath
    ?? (file as DriveFile & { path?: string }).path;
  return typeof candidate === 'string' && candidate.trim() ? candidate.trim() : null;
}

/** Collects the local absolute paths of a cut selection. */
export function resolveCutLocalPaths(selection: DriveCutSelection): string[] {
  return selection.files
    .map((file) => resolveCutLocalPath(file))
    .filter((path): path is string => path !== null);
}
