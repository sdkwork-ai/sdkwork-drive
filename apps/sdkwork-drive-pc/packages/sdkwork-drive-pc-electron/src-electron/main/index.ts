/**
 * Electron main process entry.
 *
 * The main process is a thin composition layer: it creates the window, wires the
 * tray/shortcut/clipboard controllers to the IPC registry, and owns lifecycle.
 * It holds no product logic and no SDK transport.
 *
 * Security baseline (mandatory, DESKTOP_APP_ARCHITECTURE_SPEC section 5.3):
 * `contextIsolation: true`, `nodeIntegration: false`, `sandbox: true`,
 * `webSecurity: true`; production loads the packaged renderer only.
 */

import { app, BrowserWindow, type Event as ElectronEvent } from 'electron';
import { existsSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { ELECTRON_CAPABILITIES, HOST_EVENTS } from '../shared/ipc-channels';
import { createClipboardController } from './clipboard';
import { emitHostEvent, registerIpcHandlers, unregisterIpcHandlers } from './ipc';
import { createShortcutController } from './shortcuts';
import { createTrayController, type TrayController } from './tray';
import { createMainWindow, restoreWindow } from './window';

/** Resolves the renderer target from the environment the launcher sets. */
function resolveRendererTarget():
  | { kind: 'file'; path: string }
  | { kind: 'url'; url: string } {
  const devServerUrl = process.env.ELECTRON_START_URL?.trim();
  if (devServerUrl) {
    return { kind: 'url', url: devServerUrl };
  }
  return { kind: 'file', path: path.join(resolveRendererRoot(), 'index.html') };
}

/**
 * Locates the renderer bundle.
 *
 * Packaged: electron-builder copies the app build to
 * `<resources>/renderer` (`extraResources`), so `process.resourcesPath` is the
 * anchor.
 *
 * Unpackaged: the main process runs from `dist-electron/src-electron/main`, so
 * walk back up to the package root and into the app's `dist`. The app build is
 * environment-scoped (`dist/<deploymentProfile>/<environment>/`), so the layout
 * is scanned for the first directory that actually holds an `index.html`
 * instead of assuming a fixed `dist/index.html` — a hardcoded path resolves to
 * a directory that never exists and the window would load blank.
 */
function resolveRendererRoot(): string {
  const packaged = path.join(process.resourcesPath ?? '', 'renderer');
  if (process.resourcesPath && existsSync(path.join(packaged, 'index.html'))) {
    return packaged;
  }
  const appDist = path.join(__dirname, '..', '..', '..', '..', '..', 'dist');
  const nested = findIndexHtmlRoot(appDist);
  return nested ?? appDist;
}

/** Depth-first search for the nearest directory containing an `index.html`. */
function findIndexHtmlRoot(root: string): string | null {
  if (!existsSync(root)) {
    return null;
  }
  const entries = readdirSync(root, { withFileTypes: true });
  const directories: string[] = [];
  for (const entry of entries) {
    if (!entry.isDirectory()) {
      continue;
    }
    const candidate = path.join(root, entry.name);
    if (existsSync(path.join(candidate, 'index.html'))) {
      return candidate;
    }
    directories.push(candidate);
  }
  for (const directory of directories) {
    const nested = findIndexHtmlRoot(directory);
    if (nested) {
      return nested;
    }
  }
  return null;
}

/**
 * Locates the tray icon.
 *
 * Packaged: `resources/**` is not in `files`, so electron-builder only carries
 * it through as an extra resource; the icon is therefore resolved from the app
 * resources directory. Unpackaged: read it straight from the package.
 */
function resolveIconPath(): string {
  const packaged = path.join(process.resourcesPath ?? '', 'icons', 'icon.png');
  if (process.resourcesPath && existsSync(packaged)) {
    return packaged;
  }
  return path.join(__dirname, '..', '..', '..', 'resources', 'icons', 'icon.png');
}

let mainWindow: BrowserWindow | null = null;
let tray: TrayController | null = null;

function getMainWindow(): BrowserWindow | null {
  return mainWindow;
}

function bootstrap(): void {
  mainWindow = createMainWindow({
    rendererTarget: resolveRendererTarget(),
    preloadPath: path.join(__dirname, '..', 'preload', 'index.js'),
    appOrigin: process.env.ELECTRON_START_URL?.trim() || undefined,
  });

  mainWindow.on('closed', () => {
    mainWindow = null;
  });

  tray = createTrayController({
    iconPath: resolveIconPath(),
    emitMenuActivated: (menuItemId) => {
      emitHostEvent(mainWindow, HOST_EVENTS.trayMenu, menuItemId);
    },
    restoreWindow: () => restoreWindow(mainWindow),
    tooltip: 'SDKWork Drive',
  });

  const shortcuts = createShortcutController({
    emitTriggered: (bindingId) => {
      emitHostEvent(mainWindow, HOST_EVENTS.shortcutTriggered, bindingId);
    },
  });

  const clipboard = createClipboardController();

  registerIpcHandlers({
    getMainWindow,
    tray,
    shortcuts,
    clipboard,
  });

  // The renderer queries the declared capability set during bootstrap so it can
  // route by capability instead of branching on the host identity.
  mainWindow.webContents.on('did-finish-load', () => {
    mainWindow?.webContents.send('sdkwork:host:capabilities', {
      id: 'electron',
      capabilities: [...ELECTRON_CAPABILITIES],
    });
  });
}

// Single-instance: a second launch restores the existing window instead of
// stacking a duplicate tray icon.
const gotLock = app.requestSingleInstanceLock();
if (!gotLock) {
  app.quit();
} else {
  app.on('second-instance', () => {
    restoreWindow(mainWindow);
  });

  app.whenReady().then(bootstrap).catch((error: unknown) => {
    console.error('[sdkwork-drive-pc-electron] bootstrap failed', error);
    app.quit();
  });

  app.on('window-all-closed', () => {
    // The tray keeps the app alive on Windows and Linux; macOS apps stay
    // resident by platform convention.
    if (process.platform !== 'darwin') {
      app.quit();
    }
  });

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      bootstrap();
      return;
    }
    restoreWindow(mainWindow);
  });

  app.on('will-quit', (_event: ElectronEvent) => {
    unregisterIpcHandlers();
    tray?.destroy();
    tray = null;
  });
}
