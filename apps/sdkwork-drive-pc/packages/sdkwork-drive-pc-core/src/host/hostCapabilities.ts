/**
 * Typed host capability contracts for tray, shortcuts, and clipboard cut.
 *
 * These interfaces mirror `DESKTOP_APP_ARCHITECTURE_SPEC.md` section 5.5
 * (`window | tray | ... | clipboard`) and section 5.6 (bridge protocol method
 * names `sdkwork:<capability>:<action>`). Every native host — Tauri and
 * Electron — satisfies the same interfaces; feature packages depend on these
 * types or on injected host objects and never on host globals.
 */

/** Capability identifiers owned by the desktop hosts. */
export type DesktopHostCapability =
  | 'window'
  | 'tray'
  | 'shortcuts'
  | 'clipboard'
  | 'filePicker'
  | 'filesystemSandbox'
  | 'shellOpen'
  | 'secureStorage';

/** Bridge protocol method names implemented by the native hosts. */
export const HOST_BRIDGE_METHODS = {
  traySetVisible: 'sdkwork:tray:setVisible',
  traySetMenu: 'sdkwork:tray:setMenu',
  trayEmitMenu: 'sdkwork:tray:emitMenu',
  trayRestoreWindow: 'sdkwork:tray:restoreWindow',
  trayMenuActivated: 'sdkwork:tray:menu',
  shortcutRegisterAll: 'sdkwork:shortcut:registerAll',
  shortcutUnregister: 'sdkwork:shortcut:unregister',
  shortcutUnregisterAll: 'sdkwork:shortcut:unregisterAll',
  shortcutTriggered: 'sdkwork:shortcut:triggered',
  clipboardCutPaths: 'sdkwork:clipboard:cutPaths',
  clipboardReadPaths: 'sdkwork:clipboard:readPaths',
  clipboardWriteText: 'sdkwork:clipboard:writeText',
} as const;

export type HostBridgeMethod =
  (typeof HOST_BRIDGE_METHODS)[keyof typeof HOST_BRIDGE_METHODS];

/** Stable outcome codes allowed by the host adapter contract. */
export type HostErrorCode =
  | 'unsupported'
  | 'permission-denied'
  | 'unavailable'
  | 'cancelled'
  | 'invalid-state'
  | 'internal';

export interface HostOk<T> {
  ok: true;
  value: T;
}

export interface HostErr {
  ok: false;
  error: {
    code: HostErrorCode;
    message: string;
    detail?: unknown;
  };
}

export type HostResult<T> = HostOk<T> | HostErr;

export interface TrayMenuItem {
  /** Stable command id owned by the renderer (for example `drive.openSettings`). */
  id: string;
  label: string;
  enabled: boolean;
}

export interface TraySetMenuRequest {
  items: TrayMenuItem[];
  tooltip?: string | null;
}

/** Tray capability group. */
export interface TrayCapability {
  /** Shows or hides the tray icon. */
  setVisible(visible: boolean): Promise<HostResult<void>>;
  /** Replaces the tray context menu. */
  setMenu(request: TraySetMenuRequest): Promise<HostResult<void>>;
  /** Restores, raises, and focuses the main window. */
  restoreWindow(): Promise<HostResult<void>>;
  /**
   * Subscribes to tray menu activation.
   * Returns an unsubscribe function; a no-op unsubscribe when unsupported.
   */
  onMenuActivated(listener: (menuItemId: string) => void): () => void;
}

export interface ShortcutBinding {
  /** Stable command id shared with the tray menu ids. */
  id: string;
  /** Accelerator such as `CommandOrControl+Shift+X`. */
  accelerator: string;
}

export interface ShortcutRegistrationOutcome {
  id: string;
  registered: boolean;
  reason?: string | null;
}

/** Shortcut capability group. */
export interface ShortcutCapability {
  /** Registers or rebinds a batch of accelerators, reporting per-binding outcome. */
  registerAll(bindings: ShortcutBinding[]): Promise<HostResult<ShortcutRegistrationOutcome[]>>;
  /** Unregisters one accelerator. */
  unregister(binding: ShortcutBinding): Promise<HostResult<void>>;
  /** Unregisters every accelerator owned by the host. */
  unregisterAll(): Promise<HostResult<void>>;
  /**
   * Subscribes to accelerator activation. The host emits the binding id, never
   * the raw key combination, so the renderer keeps one command dispatch table.
   */
  onTriggered(listener: (bindingId: string) => void): () => void;
}

export interface ClipboardCutRequest {
  /** Absolute local paths to place on the clipboard as a cut. */
  paths: string[];
}

export interface ClipboardCutResult {
  accepted: boolean;
  acceptedCount: number;
  reason?: string | null;
}

/** Clipboard capability group. */
export interface ClipboardCapability {
  /** Writes local paths to the OS clipboard marked as cut (move semantics). */
  cutPaths(request: ClipboardCutRequest): Promise<HostResult<ClipboardCutResult>>;
  /** Reads back the file-list payload currently on the clipboard. */
  readPaths(): Promise<HostResult<string[]>>;
  /** Writes plain text to the clipboard. */
  writeText(text: string): Promise<HostResult<void>>;
}
