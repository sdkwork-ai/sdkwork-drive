/**
 * Electron IPC registry.
 *
 * `ipcMain.handle` channels equal the bridge protocol method names and every
 * handler is wrapped so a native failure becomes a structured
 * `{ ok: false, error: { code, message } }` outcome instead of a thrown string.
 * Unknown channels are never registered, so they are rejected rather than
 * forwarded (DESKTOP_APP_ARCHITECTURE_SPEC section 5.6).
 */

import { ipcMain, type BrowserWindow } from 'electron';
import {
  ALLOWED_METHODS,
  CLIPBOARD_METHODS,
  BridgeError,
  HOST_EVENTS,
  SHELL_METHODS,
  SHORTCUT_METHODS,
  TRAY_METHODS,
  WINDOW_METHODS,
  isAllowedMethod,
} from '../shared/ipc-channels';
import type { ClipboardController, ClipboardCutParams } from './clipboard';
import type { ShortcutBindingPayload, ShortcutController } from './shortcuts';
import type { TrayController, TraySetMenuParams } from './tray';

export interface IpcRegistryOptions {
  getMainWindow: () => BrowserWindow | null;
  tray: TrayController;
  shortcuts: ShortcutController;
  clipboard: ClipboardController;
}

type InvokeHandler = (params: Record<string, unknown> | undefined) => unknown;

/**
 * Emits a host-initiated event to the renderer.
 *
 * Events are allowlisted too: an unknown event name is dropped so the host can
 * never become a generic event bus.
 */
export function emitHostEvent(
  window: BrowserWindow | null,
  event: string,
  payload: unknown,
): void {
  if (!window || window.isDestroyed() || !Object.values(HOST_EVENTS).includes(event as never)) {
    return;
  }
  window.webContents.send(event, payload);
}

function applyWindowAction(window: BrowserWindow | null, action: string): void {
  if (!window || window.isDestroyed()) {
    throw new BridgeError('unavailable', 'Main window is unavailable.');
  }
  switch (action) {
    case WINDOW_METHODS.minimize:
      window.minimize();
      return;
    case WINDOW_METHODS.maximize:
      window.maximize();
      return;
    case WINDOW_METHODS.unmaximize:
      window.unmaximize();
      return;
    case WINDOW_METHODS.close:
      window.close();
      return;
    case WINDOW_METHODS.show:
      window.show();
      return;
    default:
      throw new BridgeError('unsupported', `Unsupported window action: ${action}`);
  }
}

function buildHandlers(options: IpcRegistryOptions): Map<string, InvokeHandler> {
  const { tray, shortcuts, clipboard, getMainWindow } = options;

  return new Map<string, InvokeHandler>([
    [
      TRAY_METHODS.setVisible,
      (params) => {
        tray.setVisible(Boolean(params?.visible));
        return null;
      },
    ],
    [
      TRAY_METHODS.setMenu,
      (params) => {
        tray.setMenu((params ?? { items: [] }) as unknown as TraySetMenuParams);
        return null;
      },
    ],
    [
      TRAY_METHODS.restoreWindow,
      () => {
        const window = getMainWindow();
        if (window && !window.isDestroyed()) {
          if (window.isMinimized()) {
            window.restore();
          }
          window.show();
          window.focus();
        }
        return null;
      },
    ],
    [
      SHORTCUT_METHODS.registerAll,
      (params) => shortcuts.registerAll((params?.bindings ?? []) as ShortcutBindingPayload[]),
    ],
    [
      SHORTCUT_METHODS.unregister,
      (params) => {
        shortcuts.unregister(params as unknown as ShortcutBindingPayload);
        return null;
      },
    ],
    [
      SHORTCUT_METHODS.unregisterAll,
      () => {
        shortcuts.unregisterAll();
        return null;
      },
    ],
    [
      CLIPBOARD_METHODS.cutPaths,
      (params) => clipboard.cutPaths((params ?? { paths: [] }) as unknown as ClipboardCutParams),
    ],
    [CLIPBOARD_METHODS.readPaths, () => clipboard.readPaths()],
    [
      CLIPBOARD_METHODS.writeText,
      (params) => {
        clipboard.writeText(String(params?.text ?? ''));
        return null;
      },
    ],
    ...Object.values(WINDOW_METHODS).map(
      (method) =>
        [
          method,
          () => {
            applyWindowAction(getMainWindow(), method);
            return null;
          },
        ] as [string, InvokeHandler],
    ),
    [
      SHELL_METHODS.openExternal,
      (params) => {
        const url = String(params?.url ?? '');
        let parsed: URL;
        try {
          parsed = new URL(url);
        } catch {
          throw new BridgeError('invalid-state', 'External URL is invalid.');
        }
        if (parsed.protocol !== 'https:' && parsed.protocol !== 'http:') {
          throw new BridgeError('permission-denied', 'Only HTTP(S) URLs can be opened externally.');
        }
        void import('electron').then(({ shell }) => shell.openExternal(url));
        return null;
      },
    ],
  ]);
}

/**
 * Registers every bridge handler.
 *
 * The preload allowlist and this registry are generated from the same table, so
 * a method can never be reachable without an explicit handler.
 */
export function registerIpcHandlers(options: IpcRegistryOptions): void {
  const handlers = buildHandlers(options);

  for (const [method, handler] of handlers) {
    if (!isAllowedMethod(method)) {
      throw new BridgeError('internal', `Refusing to register non-allowlisted method: ${method}`);
    }
    ipcMain.handle(method, async (_event, params?: Record<string, unknown>) => {
      try {
        return { ok: true, value: await handler(params) };
      } catch (error) {
        if (error instanceof BridgeError) {
          return { ok: false, error: { code: error.code, message: error.message } };
        }
        return {
          ok: false,
          error: {
            code: 'internal',
            message: error instanceof Error ? error.message : String(error),
          },
        };
      }
    });
  }
}

/** Removes every bridge handler (used on shutdown and in tests). */
export function unregisterIpcHandlers(): void {
  for (const method of ALLOWED_METHODS) {
    ipcMain.removeHandler(method);
  }
}
