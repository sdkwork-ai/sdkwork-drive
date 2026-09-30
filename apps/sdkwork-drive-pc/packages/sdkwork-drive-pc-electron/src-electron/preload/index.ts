/**
 * Electron preload bridge.
 *
 * This is the ONLY renderer-to-main channel. It exposes a method allowlist
 * through `contextBridge.exposeInMainWorld` and never a generic pass-through of
 * `ipcRenderer` (DESKTOP_APP_ARCHITECTURE_SPEC section 5.3 / 5.6).
 *
 * Unknown methods and unknown events are rejected locally, so a compromised
 * renderer cannot reach an unregistered channel.
 */

import { contextBridge, ipcRenderer } from 'electron';
import {
  ALLOWED_EVENTS,
  ALLOWED_METHODS,
  ELECTRON_CAPABILITIES,
  isAllowedEvent,
  isAllowedMethod,
  type HostErrorCode,
} from '../shared/ipc-channels';

interface BridgeInvokeOutcome {
  ok: boolean;
  value?: unknown;
  error?: { code: HostErrorCode; message: string };
}

function invoke<T>(method: string, params?: Record<string, unknown>): Promise<T> {
  if (!isAllowedMethod(method)) {
    return Promise.reject(
      Object.assign(new Error(`Bridge method is not allowlisted: ${method}`), {
        code: 'unsupported' satisfies HostErrorCode,
      }),
    );
  }
  return ipcRenderer.invoke(method, params) as Promise<T>;
}

/**
 * Subscribes to a host-initiated event.
 *
 * Returns a synchronous unsubscribe, matching the host adapter contract so
 * renderer code is identical across the Tauri and Electron hosts.
 */
function on(event: string, listener: (payload: unknown) => void): () => void {
  if (!isAllowedEvent(event)) {
    return () => {};
  }
  const wrapped = (_event: unknown, payload: unknown) => listener(payload);
  ipcRenderer.on(event, wrapped);
  return () => {
    ipcRenderer.removeListener(event, wrapped);
  };
}

const sdkworkDesktop = {
  meta: {
    id: 'electron' as const,
    capabilities: [...ELECTRON_CAPABILITIES],
    allowedMethods: [...ALLOWED_METHODS],
    allowedEvents: [...ALLOWED_EVENTS],
  },
  invoke,
  on,
};

contextBridge.exposeInMainWorld('sdkworkDesktop', sdkworkDesktop);

export type SdkworkDesktopBridge = typeof sdkworkDesktop;
export type { BridgeInvokeOutcome };
