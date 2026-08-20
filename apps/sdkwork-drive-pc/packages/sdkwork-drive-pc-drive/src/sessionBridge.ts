import type { SessionStore } from 'sdkwork-drive-pc-core';

import { getDrivePcSdkPorts } from './sdkPorts';

export function syncHostSessionIntoDriveStore(session: SessionStore): void {
  const hostSession = getDrivePcSdkPorts().readHostSession();
  if (hostSession) {
    session.setSession(hostSession);
    return;
  }
  session.clearSession();
}

export function bindHostSessionToDriveStore(session: SessionStore): () => void {
  syncHostSessionIntoDriveStore(session);
  const subscribe = getDrivePcSdkPorts().subscribeHostSession;
  if (!subscribe) {
    return () => undefined;
  }
  return subscribe(() => {
    syncHostSessionIntoDriveStore(session);
  });
}
