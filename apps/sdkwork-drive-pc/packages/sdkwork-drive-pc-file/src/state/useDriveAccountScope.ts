import { useEffect, useRef, useState } from 'react';
import {
  driveStorageScopeKey,
  resolveDriveStorageScope,
  type DriveFileService,
  type DriveStorageScope,
  type SessionSnapshot,
  type SessionStore,
} from 'sdkwork-drive-pc-core';

import { setDriveCutClipboardScope } from './driveCutClipboard';

export interface DriveAccountScope {
  /** Current session snapshot (tokens, context, user). */
  snapshot: SessionSnapshot;
  /** Account that owns device-persisted Drive state, when both ids are known. */
  scope: DriveStorageScope | undefined;
  /** Stable `tenant:user` key, or '' while the account is unknown or signed out. */
  scopeKey: string;
  signedIn: boolean;
  /** Account-scoped state may be persisted only when an account owns it. */
  canPersist: boolean;
}

/**
 * Single source of truth for "which account is this renderer serving".
 *
 * The Drive runtime, the file service and the cut clipboard all outlive a
 * sign-in, so this hook invalidates their account-scoped state the moment the
 * session identity changes. It also decides whether device persistence is safe:
 * with tokens but no tenant/user context the account has no owner, and writing
 * an unscoped key would hand the data to the next account on the device.
 */
export function useDriveAccountScope(
  session: SessionStore,
  fileService: DriveFileService,
): DriveAccountScope {
  const [snapshot, setSnapshot] = useState<SessionSnapshot>(() => session.getSnapshot());
  useEffect(() => session.subscribe(setSnapshot), [session]);

  const scope = resolveDriveStorageScope(snapshot);
  const scopeKey = driveStorageScopeKey(scope);
  const signedIn = Boolean(snapshot.accessToken || snapshot.authToken || snapshot.refreshToken);

  // Invalidate the previous account's state synchronously during render: the file
  // service caches and the pending cut are read by render (space titles, cut
  // badge), so an effect would expose them for one frame after a switch. Both
  // calls are no-ops for an unchanged scope key.
  fileService.syncAccountScope(scopeKey);
  setDriveCutClipboardScope(scopeKey);

  const missingScopeWarnedRef = useRef(false);
  useEffect(() => {
    if (!signedIn || scopeKey !== '') {
      missingScopeWarnedRef.current = false;
      return;
    }
    if (missingScopeWarnedRef.current) {
      return;
    }
    missingScopeWarnedRef.current = true;
    console.warn(
      '[sdkwork-drive-pc] Signed in without tenant/user context: account-scoped state '
      + '(transfer list, uploader sessions, spaces, favorites) stays in memory and is '
      + 'not written to device storage.',
    );
  }, [scopeKey, signedIn]);

  return {
    snapshot,
    scope,
    scopeKey,
    signedIn,
    canPersist: signedIn && scopeKey !== '',
  };
}
