/**
 * Host command dispatcher.
 *
 * Single command table shared by the tray menu, the Settings Center shortcut
 * bindings, and in-app keyboard handling. The native hosts emit only a command
 * id (`sdkwork:tray:menu`, `sdkwork:shortcut:triggered`), so one table decides
 * what every command does regardless of how it was triggered.
 */

import type { DrivePcShortcutCommandId } from 'sdkwork-drive-pc-commons';

import type { HostAdapter } from '../host/hostAdapter';
import type { HostResult } from '../host/hostCapabilities';

/** Every command the desktop host can dispatch. */
export type DriveHostCommandId = DrivePcShortcutCommandId;

export interface DriveHostCommandHandlers {
  openSettings: () => void;
  showWindow: () => void;
  cutSelection: () => void | Promise<void>;
  pasteSelection: () => void | Promise<void>;
  refresh: () => void | Promise<void>;
  toggleTray: () => void | Promise<void>;
}

const COMMAND_HANDLER_KEY: Record<DriveHostCommandId, keyof DriveHostCommandHandlers> = {
  'drive.openSettings': 'openSettings',
  'drive.showWindow': 'showWindow',
  'drive.cutSelection': 'cutSelection',
  'drive.pasteSelection': 'pasteSelection',
  'drive.refresh': 'refresh',
  'drive.toggleTray': 'toggleTray',
};

/** Localized tray menu labels, resolved by the caller from i18n. */
export interface DriveTrayMenuLabels {
  openSettings: string;
  showWindow: string;
  cutSelection: string;
  pasteSelection: string;
  refresh: string;
  toggleTray: string;
}

const TRAY_MENU_ORDER: DriveHostCommandId[] = [
  'drive.showWindow',
  'drive.openSettings',
  'separator' as DriveHostCommandId,
  'drive.cutSelection',
  'drive.pasteSelection',
  'drive.refresh',
  'separator' as DriveHostCommandId,
  'drive.toggleTray',
];

/**
 * Builds the tray menu item list.
 *
 * Menu items reuse the shortcut command ids and labels declared in the Settings
 * Center, so the tray always describes exactly the configured command set.
 */
export function buildTrayMenuItems(
  labels: DriveTrayMenuLabels,
): { id: string; label: string; enabled: boolean }[] {
  const labelFor: Record<DriveHostCommandId, string> = {
    'drive.openSettings': labels.openSettings,
    'drive.showWindow': labels.showWindow,
    'drive.cutSelection': labels.cutSelection,
    'drive.pasteSelection': labels.pasteSelection,
    'drive.refresh': labels.refresh,
    'drive.toggleTray': labels.toggleTray,
  };

  return TRAY_MENU_ORDER.map((id) => {
    if ((id as string) === 'separator') {
      return { id: 'separator', label: '-', enabled: true };
    }
    return { id, label: labelFor[id], enabled: true };
  });
}

export function isDispatcheableCommand(commandId: string): commandId is DriveHostCommandId {
  return commandId in COMMAND_HANDLER_KEY;
}

/** Dispatches a command id to its handler. Unknown ids are ignored. */
export function dispatchHostCommand(
  commandId: string,
  handlers: DriveHostCommandHandlers,
): boolean {
  if (!isDispatcheableCommand(commandId)) {
    return false;
  }
  const handler = handlers[COMMAND_HANDLER_KEY[commandId]];
  void handler();
  return true;
}

/**
 * Registers every enabled binding with the native host and wires activation
 * back into the command table.
 *
 * Returns a teardown function that unsubscribes and releases the accelerators,
 * so rebinding never leaves a stale handler pointing at a dead closure.
 */
export function bindHostShortcuts(
  host: HostAdapter,
  handlers: DriveHostCommandHandlers,
  bindings: { id: DriveHostCommandId; accelerator: string }[],
): () => void {
  const enabled = bindings.filter((binding) => binding.accelerator.trim().length > 0);

  void host.shortcuts.registerAll(enabled).then((result: HostResult<unknown>) => {
    if (!result.ok) {
      // A host without global shortcut support (browser fallback, or a desktop
      // host denying the permission) degrades silently: the in-app bindings
      // below still work.
      return;
    }
  });

  const unsubscribe = host.shortcuts.onTriggered((bindingId) => {
    dispatchHostCommand(bindingId, handlers);
  });

  return () => {
    unsubscribe();
    void host.shortcuts.unregisterAll();
  };
}

/**
 * Wires tray menu activation into the command table.
 *
 * Returns an unsubscribe function so the tray listener is released when the
 * runtime unmounts.
 */
export function bindTrayMenu(
  host: HostAdapter,
  handlers: DriveHostCommandHandlers,
): () => void {
  return host.tray.onMenuActivated((menuItemId) => {
    dispatchHostCommand(menuItemId, handlers);
  });
}
