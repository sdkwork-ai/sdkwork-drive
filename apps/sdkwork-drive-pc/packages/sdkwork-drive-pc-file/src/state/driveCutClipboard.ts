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
 *
 * The pending cut belongs to the account that created it. A shared renderer can
 * serve a second sign-in, so the clipboard records the owning account and drops
 * itself as soon as the signed-in account changes - otherwise the next account's
 * paste would move nodes it cannot access (403/404) or, within one tenant, move
 * another user's files.
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
let pendingCutScopeKey: string | undefined;
let currentScopeKey: string | undefined;
const listeners = new Set<(selection: DriveCutSelection | null) => void>();

function notify(): void {
  for (const listener of listeners) {
    listener(pendingCut);
  }
}

/**
 * Publish a reset after the caller's render pass.
 *
 * `setDriveCutClipboardScope` runs while a component renders; notifying
 * subscribers synchronously would update other components from inside their
 * sibling's render.
 */
function scheduleNotify(): void {
  if (typeof queueMicrotask === 'function') {
    queueMicrotask(notify);
    return;
  }
  notify();
}

/**
 * Declare which account is signed in.
 *
 * Idempotent for an unchanged key, so it is safe to call on every render before
 * any reader asks for the pending cut.
 */
export function setDriveCutClipboardScope(scopeKey: string | undefined): void {
  if (currentScopeKey === scopeKey) {
    return;
  }
  currentScopeKey = scopeKey;
  if (pendingCut !== null && pendingCutScopeKey !== scopeKey) {
    pendingCut = null;
    pendingCutScopeKey = undefined;
    scheduleNotify();
  }
}

/** The account key the pending cut belongs to, or undefined when nothing is cut. */
export function getDriveCutClipboardScopeKey(): string | undefined {
  return pendingCut === null ? undefined : pendingCutScopeKey;
}

/** Reads the current pending cut, or null when nothing is cut for this account. */
export function getDriveCutSelection(): DriveCutSelection | null {
  if (pendingCut !== null && pendingCutScopeKey !== currentScopeKey) {
    // Defensive: a reader must never observe another account's cut.
    return null;
  }
  return pendingCut;
}

/** Records a pending cut and notifies subscribers. */
export function setDriveCutSelection(selection: DriveCutSelection | null): void {
  pendingCut = selection;
  pendingCutScopeKey = selection === null ? undefined : currentScopeKey;
  notify();
}

/** Clears the pending cut (after a successful move, or on Escape). */
export function clearDriveCutSelection(): void {
  if (pendingCut === null) {
    return;
  }
  pendingCut = null;
  pendingCutScopeKey = undefined;
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
