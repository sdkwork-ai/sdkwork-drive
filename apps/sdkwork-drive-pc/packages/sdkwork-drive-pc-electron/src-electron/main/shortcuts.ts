/**
 * Electron global shortcut capability.
 *
 * Mirrors the Tauri host: the renderer owns the binding list, the native host
 * registers accelerators and emits `sdkwork:shortcut:triggered` with the
 * binding id so renderer code keeps one command dispatch table.
 */

import { globalShortcut } from 'electron';
import { HOST_EVENTS, BridgeError } from '../shared/ipc-channels';

export interface ShortcutBindingPayload {
  id: string;
  accelerator: string;
}

export interface ShortcutRegistrationOutcome {
  id: string;
  registered: boolean;
  reason?: string;
}

/** Normalizes renderer accelerators to Electron's accepted spelling. */
function normalizeAccelerator(accelerator: string): string {
  const trimmed = accelerator.trim();
  if (!trimmed) {
    throw new BridgeError('invalid-state', 'Shortcut accelerator is empty.');
  }
  return trimmed
    .replace(/\bCommandOrControl\b/gi, 'CommandOrControl')
    .replace(/\bCtrl\b/gi, 'Control')
    .replace(/\bCmd\b/gi, 'Command')
    .replace(/\bOption\b/gi, 'Alt');
}

export interface ShortcutController {
  registerAll(bindings: ShortcutBindingPayload[]): ShortcutRegistrationOutcome[];
  unregister(binding: ShortcutBindingPayload): void;
  unregisterAll(): void;
  readonly registered: ReadonlyMap<string, string>;
}

export interface ShortcutControllerOptions {
  /** Emits `sdkwork:shortcut:triggered` with the binding id. */
  emitTriggered: (bindingId: string) => void;
}

export function createShortcutController(
  options: ShortcutControllerOptions,
): ShortcutController {
  const registered = new Map<string, string>();

  return {
    get registered() {
      return registered;
    },
    registerAll(bindings) {
      return bindings.map((binding) => {
        let accelerator: string;
        try {
          accelerator = normalizeAccelerator(binding.accelerator);
        } catch (error) {
          return {
            id: binding.id,
            registered: false,
            reason: error instanceof Error ? error.message : String(error),
          };
        }

        // Rebinding the same accelerator must replace the previous handler so
        // the Settings Center never leaves two commands on one key.
        if (globalShortcut.isRegistered(accelerator)) {
          globalShortcut.unregister(accelerator);
        }

        try {
          const ok = globalShortcut.register(accelerator, () => {
            options.emitTriggered(binding.id);
          });
          if (!ok) {
            return {
              id: binding.id,
              registered: false,
              reason: `Shortcut accelerator is unavailable: ${accelerator}`,
            };
          }
        } catch (error) {
          return {
            id: binding.id,
            registered: false,
            reason: error instanceof Error ? error.message : String(error),
          };
        }

        // Drop any previous accelerator bound to the same command id.
        const previous = registered.get(binding.id);
        if (previous && previous !== accelerator) {
          globalShortcut.unregister(previous);
        }
        registered.set(binding.id, accelerator);

        return { id: binding.id, registered: true };
      });
    },
    unregister(binding) {
      const accelerator = registered.get(binding.id) ?? binding.accelerator;
      if (globalShortcut.isRegistered(accelerator)) {
        globalShortcut.unregister(accelerator);
      }
      registered.delete(binding.id);
    },
    unregisterAll() {
      globalShortcut.unregisterAll();
      registered.clear();
    },
  };
}

export { HOST_EVENTS };
