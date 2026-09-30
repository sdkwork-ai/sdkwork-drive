import type { LocalFilesystemEntry } from '../types';
import type { NativeLocalUploadDescriptor } from './nativeLocalUploadFile';
import {
  HOST_BRIDGE_METHODS,
  type ClipboardCapability,
  type ClipboardCutRequest,
  type ClipboardCutResult,
  type DesktopHostCapability,
  type HostErrorCode,
  type HostResult,
  type ShortcutBinding,
  type ShortcutCapability,
  type ShortcutRegistrationOutcome,
  type TrayCapability,
  type TraySetMenuRequest,
} from './hostCapabilities';

export type WindowControlAction = 'minimize' | 'maximize' | 'unmaximize' | 'close' | 'show';

export interface HostAdapter {
  isNativeHost: boolean;
  /** Which native host is active; `browser` when no native host is present. */
  hostId: 'tauri' | 'electron' | 'browser';
  /** Declared capability set; empty for the browser fallback. */
  capabilities: ReadonlySet<DesktopHostCapability>;
  hasCapability(capability: DesktopHostCapability): boolean;
  tray: TrayCapability;
  shortcuts: ShortcutCapability;
  clipboard: ClipboardCapability;
  windowControl(action: WindowControlAction): Promise<void>;
  openExternal(url: string): Promise<void>;
  writeTextToClipboard(text: string): Promise<void>;
  listLocalFilesystem(path?: string | null): Promise<LocalFilesystemEntry[]>;
  openLocalPath(path: string): Promise<void>;
  pickLocalUploadFiles(): Promise<NativeLocalUploadDescriptor[]>;
  describeLocalUploadFile(path: string): Promise<NativeLocalUploadDescriptor>;
  readLocalUploadRange(path: string, offsetBytes: number, lengthBytes: number): Promise<ArrayBuffer>;
  checksumLocalUploadFile(path: string): Promise<string>;
  saveDownloadFile(fileName: string, blob: Blob): Promise<boolean>;
  beginDownloadSave(fileName: string): Promise<string | null>;
  writeDownloadChunk(sessionId: string, chunk: Uint8Array): Promise<void>;
  finishDownloadSave(sessionId: string): Promise<boolean>;
  abortDownloadSave(sessionId: string): Promise<void>;
}

interface TauriGlobal {
  core?: {
    invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
  };
  shell?: {
    open(url: string): Promise<void>;
  };
  clipboard?: {
    writeText(text: string): Promise<void>;
  };
  event?: {
    listen<T>(event: string, handler: (event: { payload: T }) => void): Promise<() => void>;
  };
}

/**
 * Electron preload bridge shape.
 *
 * The Electron host exposes exactly this surface through
 * `contextBridge.exposeInMainWorld('sdkworkDesktop', ...)`; the preload allowlist
 * is generated from `HOST_BRIDGE_METHODS`, so unknown channels are rejected
 * rather than forwarded (DESKTOP_APP_ARCHITECTURE_SPEC section 5.6).
 */
interface ElectronDesktopBridge {
  meta?: {
    id?: string;
    capabilities?: string[];
  };
  invoke<T>(method: string, params?: Record<string, unknown>): Promise<T>;
  on(event: string, listener: (payload: unknown) => void): () => void;
}

function getTauriGlobal(): TauriGlobal | undefined {
  return (globalThis as typeof globalThis & { __TAURI__?: TauriGlobal }).__TAURI__;
}

function getElectronBridge(): ElectronDesktopBridge | undefined {
  return (
    globalThis as typeof globalThis & { sdkworkDesktop?: ElectronDesktopBridge }
  ).sdkworkDesktop;
}

function resolveHostId(): 'tauri' | 'electron' | 'browser' {
  if (getTauriGlobal()?.core?.invoke) {
    return 'tauri';
  }
  if (getElectronBridge()?.invoke) {
    return 'electron';
  }
  return 'browser';
}

const TAURI_CAPABILITIES: DesktopHostCapability[] = [
  'window',
  'tray',
  'shortcuts',
  'clipboard',
  'filePicker',
  'filesystemSandbox',
  'shellOpen',
  'secureStorage',
];

const ELECTRON_CAPABILITIES: DesktopHostCapability[] = [
  'window',
  'tray',
  'shortcuts',
  'clipboard',
  'filePicker',
  'filesystemSandbox',
  'shellOpen',
  'secureStorage',
];

function resolveCapabilities(
  hostId: 'tauri' | 'electron' | 'browser',
): ReadonlySet<DesktopHostCapability> {
  if (hostId === 'tauri') {
    return new Set(TAURI_CAPABILITIES);
  }
  if (hostId === 'electron') {
    const declared = getElectronBridge()?.meta?.capabilities;
    return new Set(
      (declared as DesktopHostCapability[] | undefined) ?? ELECTRON_CAPABILITIES,
    );
  }
  return new Set<DesktopHostCapability>();
}

export function hostOk<T>(value: T): HostResult<T> {
  return { ok: true, value };
}

export function hostErr<T = never>(code: HostErrorCode, message: string): HostResult<T> {
  return { ok: false, error: { code, message } };
}

function hostUnsupported<T>(capability: string): HostResult<T> {
  return hostErr('unsupported', `${capability} is only available in the desktop app.`);
}

/**
 * Subscribes to a Tauri-host event, returning a synchronous unsubscribe.
 *
 * Tauri resolves listeners asynchronously, so the returned function defers the
 * real teardown until the listener handle exists. This keeps the capability
 * contract (`() => void`) stable across hosts.
 */
function listenTauriEvent<T>(event: string, listener: (payload: T) => void): () => void {
  const tauri = getTauriGlobal();
  if (!tauri?.event?.listen) {
    return () => {};
  }
  let disposed = false;
  let unlisten: (() => void) | undefined;

  void tauri.event
    .listen<T>(event, (incoming) => {
      if (!disposed) {
        listener(incoming.payload);
      }
    })
    .then((off) => {
      if (disposed) {
        off();
        return;
      }
      unlisten = off;
    })
    .catch(() => {
      // Listener registration is best-effort: a missing host event bus must not
      // break renderer boot.
    });

  return () => {
    disposed = true;
    unlisten?.();
  };
}

function createTrayCapability(): TrayCapability {
  return {
    async setVisible(visible) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        return hostUnsupported('System tray');
      }
      try {
        await tauri.core.invoke('tray_set_visible', { request: { visible } });
        return hostOk(undefined);
      } catch (error) {
        return hostErr('internal', toMessage(error));
      }
    },
    async setMenu(request: TraySetMenuRequest) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        return hostUnsupported('System tray');
      }
      try {
        await tauri.core.invoke('tray_set_menu', {
          request: {
            items: request.items.map((item) => ({
              id: item.id,
              label: item.label,
              enabled: item.enabled,
            })),
            tooltip: request.tooltip ?? null,
          },
        });
        return hostOk(undefined);
      } catch (error) {
        return hostErr('internal', toMessage(error));
      }
    },
    async restoreWindow() {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        return hostUnsupported('System tray');
      }
      try {
        await tauri.core.invoke('tray_restore_window');
        return hostOk(undefined);
      } catch (error) {
        return hostErr('internal', toMessage(error));
      }
    },
    onMenuActivated(listener) {
      return listenTauriEvent<string>(HOST_BRIDGE_METHODS.trayMenuActivated, listener);
    },
  };
}

function createShortcutCapability(): ShortcutCapability {
  return {
    async registerAll(bindings: ShortcutBinding[]) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        return hostUnsupported('Global shortcuts');
      }
      try {
        const outcomes = await tauri.core.invoke<ShortcutRegistrationOutcome[]>(
          'shortcut_register_all',
          {
            request: {
              bindings: bindings.map((binding) => ({
                id: binding.id,
                accelerator: binding.accelerator,
              })),
            },
          },
        );
        return hostOk(outcomes);
      } catch (error) {
        return hostErr('internal', toMessage(error));
      }
    },
    async unregister(binding: ShortcutBinding) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        return hostUnsupported('Global shortcuts');
      }
      try {
        await tauri.core.invoke('shortcut_unregister', {
          request: { id: binding.id, accelerator: binding.accelerator },
        });
        return hostOk(undefined);
      } catch (error) {
        return hostErr('internal', toMessage(error));
      }
    },
    async unregisterAll() {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        return hostUnsupported('Global shortcuts');
      }
      try {
        await tauri.core.invoke('shortcut_unregister_all');
        return hostOk(undefined);
      } catch (error) {
        return hostErr('internal', toMessage(error));
      }
    },
    onTriggered(listener) {
      return listenTauriEvent<string>(HOST_BRIDGE_METHODS.shortcutTriggered, listener);
    },
  };
}

function createClipboardCapability(): ClipboardCapability {
  return {
    async cutPaths(request: ClipboardCutRequest) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        return hostUnsupported('Clipboard cut');
      }
      try {
        const result = await tauri.core.invoke<ClipboardCutResult>('clipboard_cut_paths', {
          request: { paths: request.paths },
        });
        return hostOk(result);
      } catch (error) {
        return hostErr('internal', toMessage(error));
      }
    },
    async readPaths() {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        return hostUnsupported('Clipboard');
      }
      try {
        const paths = await tauri.core.invoke<string[]>('clipboard_read_paths');
        return hostOk(paths);
      } catch (error) {
        return hostErr('internal', toMessage(error));
      }
    },
    async writeText(text: string) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        return hostUnsupported('Clipboard');
      }
      try {
        await tauri.core.invoke('clipboard_write_text', { request: { text } });
        return hostOk(undefined);
      } catch (error) {
        return hostErr('internal', toMessage(error));
      }
    },
  };
}

function toMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function assertSafeExternalUrl(url: string): void {
  const parsed = new URL(url);
  if (parsed.protocol !== 'https:' && parsed.protocol !== 'http:') {
    throw new Error('Only HTTP(S) URLs can be opened externally.');
  }
}

function unsupportedNativeUploadOperation(operation: string): never {
  throw new Error(`Native upload ${operation} is only available in the desktop app.`);
}

function unsupportedNativeDownloadOperation(operation: string): never {
  throw new Error(`Native download ${operation} is only available in the desktop app.`);
}

export function createHostAdapter(): HostAdapter {
  const hostId = resolveHostId();
  const capabilities = resolveCapabilities(hostId);

  return {
    hostId,
    capabilities,
    hasCapability(capability) {
      return capabilities.has(capability);
    },
    tray: createTrayCapability(),
    shortcuts: createShortcutCapability(),
    clipboard: createClipboardCapability(),
    get isNativeHost() {
      return resolveHostId() !== 'browser';
    },
    async windowControl(action) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        return;
      }
      await tauri.core.invoke('window_control', { request: { action } });
    },
    async openExternal(url) {
      assertSafeExternalUrl(url);
      const tauri = getTauriGlobal();
      if (tauri?.shell?.open) {
        await tauri.shell.open(url);
        return;
      }
      globalThis.open?.(url, '_blank', 'noopener,noreferrer');
    },
    async writeTextToClipboard(text) {
      const tauri = getTauriGlobal();
      if (tauri?.clipboard?.writeText) {
        await tauri.clipboard.writeText(text);
        return;
      }
      await navigator.clipboard?.writeText(text);
    },
    async listLocalFilesystem(path) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        return [];
      }
      return tauri.core.invoke<LocalFilesystemEntry[]>('local_filesystem_list', {
        request: {
          path: path ?? null,
        },
      });
    },
    async openLocalPath(path) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        throw new Error('Local filesystem access is only available in the desktop app.');
      }
      await tauri.core.invoke('local_filesystem_open', {
        request: { path },
      });
    },
    async pickLocalUploadFiles() {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        return [];
      }
      return tauri.core.invoke<NativeLocalUploadDescriptor[]>('local_upload_pick_files');
    },
    async describeLocalUploadFile(path) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        unsupportedNativeUploadOperation('describe');
      }
      return tauri.core.invoke<NativeLocalUploadDescriptor>('local_upload_describe_file', {
        request: { path },
      });
    },
    async readLocalUploadRange(path, offsetBytes, lengthBytes) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        unsupportedNativeUploadOperation('read');
      }
      const response = await tauri.core.invoke<{ bytes: number[] }>('local_upload_read_range', {
        request: {
          path,
          offsetBytes,
          lengthBytes,
        },
      });
      return Uint8Array.from(response.bytes).buffer;
    },
    async checksumLocalUploadFile(path) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        unsupportedNativeUploadOperation('checksum');
      }
      const response = await tauri.core.invoke<{ checksumSha256Hex: string }>('local_upload_checksum_file', {
        request: { path },
      });
      return response.checksumSha256Hex;
    },
    async saveDownloadFile(fileName, blob) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        throw new Error('Native download save is only available in the desktop app.');
      }
      const bytes = Array.from(new Uint8Array(await blob.arrayBuffer()));
      const response = await tauri.core.invoke<{ saved: boolean }>('local_download_save', {
        request: {
          fileName,
          bytes,
        },
      });
      return response.saved;
    },
    async beginDownloadSave(fileName) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        unsupportedNativeDownloadOperation('begin');
      }
      const response = await tauri.core.invoke<{ sessionId: string; saved: boolean }>('local_download_begin', {
        request: { fileName },
      });
      return response.saved ? response.sessionId : null;
    },
    async writeDownloadChunk(sessionId, chunk) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        unsupportedNativeDownloadOperation('write');
      }
      await tauri.core.invoke('local_download_write_chunk', {
        request: {
          sessionId,
          bytes: Array.from(chunk),
        },
      });
    },
    async finishDownloadSave(sessionId) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        unsupportedNativeDownloadOperation('finish');
      }
      const response = await tauri.core.invoke<{ saved: boolean }>('local_download_finish', {
        request: { sessionId },
      });
      return response.saved;
    },
    async abortDownloadSave(sessionId) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        unsupportedNativeDownloadOperation('abort');
      }
      await tauri.core.invoke('local_download_abort', {
        request: { sessionId },
      });
    },
  };
}
