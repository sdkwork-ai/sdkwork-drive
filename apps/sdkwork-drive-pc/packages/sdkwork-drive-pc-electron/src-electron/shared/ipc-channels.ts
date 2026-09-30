/**
 * Bridge protocol channel table for the Electron host.
 *
 * Per `DESKTOP_APP_ARCHITECTURE_SPEC.md` section 5.6, `ipcMain.handle`
 * channels equal the protocol method names and the preload exposes an allowlist
 * generated from this table. Unknown channels MUST be rejected, not forwarded —
 * `isAllowedMethod` is the single gate both the preload and main process use.
 */

export const TRAY_METHODS = {
  setVisible: 'sdkwork:tray:setVisible',
  setMenu: 'sdkwork:tray:setMenu',
  restoreWindow: 'sdkwork:tray:restoreWindow',
} as const;

export const SHORTCUT_METHODS = {
  registerAll: 'sdkwork:shortcut:registerAll',
  unregister: 'sdkwork:shortcut:unregister',
  unregisterAll: 'sdkwork:shortcut:unregisterAll',
} as const;

export const CLIPBOARD_METHODS = {
  cutPaths: 'sdkwork:clipboard:cutPaths',
  readPaths: 'sdkwork:clipboard:readPaths',
  writeText: 'sdkwork:clipboard:writeText',
} as const;

export const WINDOW_METHODS = {
  minimize: 'sdkwork:window:minimize',
  maximize: 'sdkwork:window:maximize',
  unmaximize: 'sdkwork:window:unmaximize',
  close: 'sdkwork:window:close',
  show: 'sdkwork:window:show',
} as const;

export const SHELL_METHODS = {
  openExternal: 'sdkwork:shellOpen:openExternal',
} as const;

/** Host-initiated events pushed to the renderer. */
export const HOST_EVENTS = {
  trayMenu: 'sdkwork:tray:menu',
  shortcutTriggered: 'sdkwork:shortcut:triggered',
  deepLinkOpen: 'sdkwork:deepLinks:open',
} as const;

/** Every channel the main process registers and the preload forwards. */
export const ALLOWED_METHODS: readonly string[] = Object.freeze([
  ...Object.values(TRAY_METHODS),
  ...Object.values(SHORTCUT_METHODS),
  ...Object.values(CLIPBOARD_METHODS),
  ...Object.values(WINDOW_METHODS),
  ...Object.values(SHELL_METHODS),
]);

export const ALLOWED_EVENTS: readonly string[] = Object.freeze([
  ...Object.values(HOST_EVENTS),
]);

export function isAllowedMethod(method: string): boolean {
  return ALLOWED_METHODS.includes(method);
}

export function isAllowedEvent(event: string): boolean {
  return ALLOWED_EVENTS.includes(event);
}

/** Stable outcome codes allowed by the host adapter contract. */
export type HostErrorCode =
  | 'unsupported'
  | 'permission-denied'
  | 'unavailable'
  | 'cancelled'
  | 'invalid-state'
  | 'internal';

/** Structured error the preload converts into a `{ ok: false }` outcome. */
export class BridgeError extends Error {
  readonly code: HostErrorCode;

  constructor(code: HostErrorCode, message: string) {
    super(message);
    this.name = 'BridgeError';
    this.code = code;
  }
}

/** Capability set declared to the renderer through `window.sdkworkDesktop.meta`. */
export const ELECTRON_CAPABILITIES: readonly string[] = Object.freeze([
  'window',
  'tray',
  'shortcuts',
  'clipboard',
  'filePicker',
  'filesystemSandbox',
  'shellOpen',
  'secureStorage',
]);
