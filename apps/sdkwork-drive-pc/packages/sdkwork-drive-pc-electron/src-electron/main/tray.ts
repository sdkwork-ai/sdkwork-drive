/**
 * Electron tray capability.
 *
 * Owns tray lifecycle and menu construction only. Menu activation is forwarded
 * to the renderer as `sdkwork:tray:menu`, so product decisions (open settings,
 * toggle visibility, quit) stay in the renderer's command table.
 */

import { Menu, Tray, nativeImage, type MenuItemConstructorOptions } from 'electron';
import path from 'node:path';
import { HOST_EVENTS, BridgeError } from '../shared/ipc-channels';

export interface TrayMenuItemPayload {
  id: string;
  label: string;
  enabled: boolean;
}

export interface TraySetMenuParams {
  items: TrayMenuItemPayload[];
  tooltip?: string | null;
}

export interface TrayController {
  setVisible(visible: boolean): void;
  setMenu(params: TraySetMenuParams): void;
  destroy(): void;
  readonly tray: Tray | null;
}

export interface TrayControllerOptions {
  /** Absolute path to the tray icon resource. */
  iconPath: string;
  /** Emits `sdkwork:tray:menu` with the activated menu item id. */
  emitMenuActivated: (menuItemId: string) => void;
  /** Restores and focuses the main window; wired by the host index. */
  restoreWindow: () => void;
  tooltip?: string;
}

function isSeparator(item: TrayMenuItemPayload): boolean {
  return item.id === 'separator' || item.id === '-';
}

function toMenuTemplate(
  items: TrayMenuItemPayload[],
  emitMenuActivated: (menuItemId: string) => void,
): MenuItemConstructorOptions[] {
  return items.map((item) => {
    if (isSeparator(item)) {
      return { type: 'separator' as const };
    }
    return {
      id: item.id,
      label: item.label,
      enabled: item.enabled,
      click: () => emitMenuActivated(item.id),
    };
  });
}

export function createTrayController(options: TrayControllerOptions): TrayController {
  const image = nativeImage.createFromPath(path.resolve(options.iconPath));
  if (image.isEmpty()) {
    throw new BridgeError('unavailable', 'Tray icon resource could not be loaded.');
  }

  const tray = new Tray(image);
  tray.setToolTip(options.tooltip ?? 'SDKWork Drive');

  // Left click restores the window; the menu opens on right click so a single
  // click never traps the user inside a menu.
  tray.on('click', () => options.restoreWindow());

  return {
    get tray() {
      return tray;
    },
    setVisible(visible: boolean) {
      if (visible) {
        tray.setImage(image);
        tray.setToolTip(options.tooltip ?? 'SDKWork Drive');
        return;
      }
      // Electron has no `hide`; an empty image removes the icon from the tray.
      tray.setImage(nativeImage.createEmpty());
    },
    setMenu(params: TraySetMenuParams) {
      if (params.tooltip) {
        tray.setToolTip(params.tooltip);
      }
      const menu = Menu.buildFromTemplate(
        toMenuTemplate(params.items, options.emitMenuActivated),
      );
      tray.setContextMenu(menu);
    },
    destroy() {
      tray.destroy();
    },
  };
}

export { HOST_EVENTS };
