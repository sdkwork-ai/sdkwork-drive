/**
 * Desktop host effects hook.
 *
 * Mounts the tray menu, the global shortcut bindings, and the in-app fallback
 * key handling for the active host. Mounted once from the root shell so every
 * host effect has exactly one owner.
 *
 * Degradation: when no native host is present the hook still installs the in-app
 * key handler, so browser builds keep the same command surface for the keys the
 * browser can observe.
 */

import { useCallback, useEffect, useMemo, useRef } from 'react';
import {
  DEFAULT_SHORTCUT_BINDINGS,
  type DrivePcPreferences,
  type DrivePcShortcutCommandId,
} from 'sdkwork-drive-pc-commons';

import type { HostAdapter } from '../host/hostAdapter';
import {
  bindHostShortcuts,
  bindTrayMenu,
  buildTrayMenuItems,
  dispatchHostCommand,
  type DriveHostCommandHandlers,
  type DriveTrayMenuLabels,
} from '../host/hostCommands';

export interface UseDriveHostEffectsOptions {
  host: HostAdapter;
  preferences: DrivePcPreferences;
  handlers: DriveHostCommandHandlers;
  trayLabels: DriveTrayMenuLabels;
}

/** Maps a browser KeyboardEvent to the accelerator spelling hosts accept. */
function acceleratorFromBrowserEvent(event: KeyboardEvent): string | null {
  const token = (() => {
    const { code, key } = event;
    if (code.startsWith('Key')) return code.slice(3);
    if (code.startsWith('Digit')) return code.slice(5);
    if (/^F\d{1,2}$/.test(code)) return code;
    const named: Record<string, string> = {
      Comma: ',',
      Period: '.',
      Slash: '/',
      Semicolon: ';',
      Quote: "'",
      BracketLeft: '[',
      BracketRight: ']',
      Backslash: '\\',
      Minus: '-',
      Equal: '=',
      Backquote: '`',
      Space: 'Space',
      Enter: 'Enter',
      Tab: 'Tab',
      Backspace: 'Backspace',
      Delete: 'Delete',
      ArrowUp: 'Up',
      ArrowDown: 'Down',
      ArrowLeft: 'Left',
      ArrowRight: 'Right',
    };
    if (named[code]) return named[code];
    return key.length === 1 ? key.toUpperCase() : null;
  })();

  if (!token) {
    return null;
  }

  const parts: string[] = [];
  if (event.ctrlKey || event.metaKey) parts.push('CommandOrControl');
  if (event.altKey) parts.push('Alt');
  if (event.shiftKey) parts.push('Shift');
  parts.push(token);
  return parts.join('+');
}

export function useDriveHostEffects({
  host,
  preferences,
  handlers,
  trayLabels,
}: UseDriveHostEffectsOptions): void {
  // Handlers are recreated per render; keeping them in a ref prevents the
  // native listeners from being re-registered on every state change.
  const handlersRef = useRef(handlers);
  handlersRef.current = handlers;

  const stableHandlers = useMemo<DriveHostCommandHandlers>(
    () => ({
      openSettings: () => handlersRef.current.openSettings(),
      showWindow: () => handlersRef.current.showWindow(),
      cutSelection: () => handlersRef.current.cutSelection(),
      pasteSelection: () => handlersRef.current.pasteSelection(),
      refresh: () => handlersRef.current.refresh(),
      toggleTray: () => handlersRef.current.toggleTray(),
    }),
    [],
  );

  const { globalShortcutsEnabled, trayEnabled, shortcutBindings } = preferences;

  // Tray icon visibility follows the preference.
  useEffect(() => {
    if (!host.hasCapability('tray')) {
      return;
    }
    void host.tray.setVisible(trayEnabled);
  }, [host, trayEnabled]);

  // Tray menu contents follow the localized labels.
  useEffect(() => {
    if (!host.hasCapability('tray') || !trayEnabled) {
      return;
    }
    const items = buildTrayMenuItems(trayLabels);
    void host.tray.setMenu({
      items,
      tooltip: trayLabels.showWindow,
    });
  }, [
    host,
    trayEnabled,
    trayLabels.openSettings,
    trayLabels.showWindow,
    trayLabels.cutSelection,
    trayLabels.pasteSelection,
    trayLabels.refresh,
    trayLabels.toggleTray,
    trayLabels,
  ]);

  // Tray menu activation -> command table.
  useEffect(() => {
    if (!host.hasCapability('tray')) {
      return;
    }
    return bindTrayMenu(host, stableHandlers);
  }, [host, stableHandlers]);

  // Global shortcut registration follows the enabled flag and the bindings.
  useEffect(() => {
    if (!host.hasCapability('shortcuts') || !globalShortcutsEnabled) {
      if (host.hasCapability('shortcuts')) {
        void host.shortcuts.unregisterAll();
      }
      return;
    }

    const bindings = (Object.keys(DEFAULT_SHORTCUT_BINDINGS) as DrivePcShortcutCommandId[]).map(
      (id) => ({ id, accelerator: shortcutBindings[id] ?? '' }),
    );

    return bindHostShortcuts(host, stableHandlers, bindings);
  }, [host, globalShortcutsEnabled, shortcutBindings, stableHandlers]);

  // In-app key handling.
  //
  // Desktop hosts own OS-level accelerators, so the renderer only intercepts
  // keys the browser can see: this keeps the browser build functional and gives
  // the desktop build a same-document fallback when a global binding conflicts
  // with another application.
  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      const accelerator = acceleratorFromBrowserEvent(event);
      if (!accelerator) {
        return;
      }

      const commands = Object.keys(DEFAULT_SHORTCUT_BINDINGS) as DrivePcShortcutCommandId[];
      for (const commandId of commands) {
        const bound = shortcutBindings[commandId];
        if (!bound || bound !== accelerator) {
          continue;
        }
        // Only intercept commands that are meaningful in the renderer here;
        // global-only commands would double-fire when the native host already
        // handled them.
        if (!event.ctrlKey && !event.metaKey) {
          continue;
        }
        event.preventDefault();
        dispatchHostCommand(commandId, handlersRef.current);
        return;
      }
    },
    [shortcutBindings],
  );

  useEffect(() => {
    if (host.hostId !== 'browser') {
      // Native hosts own the accelerators; the renderer must not double-handle
      // the same combination.
      return;
    }
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [host.hostId, handleKeyDown]);
}
