/**
 * Electron main-window model.
 *
 * Window options mirror the Tauri window profile declared in
 * `src-tauri/tauri.conf.json` (1280x800, min 1024x680, resizable, decorated,
 * centered) so both hosts present the same window to the user.
 *
 * The security baseline from DESKTOP_APP_ARCHITECTURE_SPEC section 5.3 is
 * mandatory and is intentionally not parameterized:
 * `contextIsolation: true`, `nodeIntegration: false`, `sandbox: true`,
 * `webSecurity: true`.
 */

import { BrowserWindow, shell, type BrowserWindowConstructorOptions } from 'electron';
import path from 'node:path';

export interface MainWindowOptions {
  /** Packaged renderer entry (production) or dev server URL (development). */
  rendererTarget: { kind: 'file'; path: string } | { kind: 'url'; url: string };
  preloadPath: string;
  /** Only http(s) origins may be opened externally. */
  appOrigin?: string;
}

export const MAIN_WINDOW_DIMENSIONS = Object.freeze({
  width: 1280,
  height: 800,
  minWidth: 1024,
  minHeight: 680,
});

function assertSafeExternalUrl(url: string): boolean {
  try {
    const parsed = new URL(url);
    return parsed.protocol === 'https:' || parsed.protocol === 'http:';
  } catch {
    return false;
  }
}

function buildWindowOptions(
  options: MainWindowOptions,
): BrowserWindowConstructorOptions {
  return {
    ...MAIN_WINDOW_DIMENSIONS,
    title: 'SDKWork Drive',
    resizable: true,
    frame: true,
    center: true,
    show: false,
    backgroundColor: '#111111',
    webPreferences: {
      preload: path.resolve(options.preloadPath),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
      webSecurity: true,
    },
  };
}

export function createMainWindow(options: MainWindowOptions): BrowserWindow {
  const window = new BrowserWindow(buildWindowOptions(options));

  if (options.rendererTarget.kind === 'file') {
    void window.loadFile(path.resolve(options.rendererTarget.path));
  } else {
    void window.loadURL(options.rendererTarget.url);
  }

  window.once('ready-to-show', () => {
    window.show();
  });

  // Navigation policy: external links leave the app, in-app navigation stays.
  window.webContents.setWindowOpenHandler(({ url }) => {
    if (assertSafeExternalUrl(url)) {
      void shell.openExternal(url);
    }
    return { action: 'deny' };
  });

  window.webContents.on('will-navigate', (event, url) => {
    if (options.appOrigin && !url.startsWith(options.appOrigin)) {
      event.preventDefault();
      if (assertSafeExternalUrl(url)) {
        void shell.openExternal(url);
      }
    }
  });

  return window;
}

/** Shows, unminimizes, and focuses a window (used by the tray left click). */
export function restoreWindow(window: BrowserWindow | null): void {
  if (!window || window.isDestroyed()) {
    return;
  }
  if (window.isMinimized()) {
    window.restore();
  }
  window.show();
  window.focus();
}
