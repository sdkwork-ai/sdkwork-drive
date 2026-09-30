/**
 * Renderer-side Electron `DesktopHost` adapter.
 *
 * Satisfies the same contract as the Tauri adapter (`hostCapabilities.ts`) by
 * translating capability calls into bridge protocol methods
 * (`sdkwork:<capability>:<action>`) over the preload bridge.
 *
 * Feature packages never import this module directly; the host registry in
 * `pc-core/src/host/` selects the active adapter at bootstrap.
 */

import {
  HOST_BRIDGE_METHODS,
  type ClipboardCapability,
  type ClipboardCutRequest,
  type ClipboardCutResult,
  type DesktopHostCapability,
  type HostResult,
  type ShortcutBinding,
  type ShortcutCapability,
  type ShortcutRegistrationOutcome,
  type TrayCapability,
  type TraySetMenuRequest,
} from 'sdkwork-drive-pc-core/host';

interface ElectronBridgeShape {
  meta?: {
    id?: string;
    capabilities?: string[];
  };
  invoke<T>(method: string, params?: Record<string, unknown>): Promise<T>;
  on(event: string, listener: (payload: unknown) => void): () => void;
}

function getBridge(): ElectronBridgeShape | undefined {
  return (
    globalThis as typeof globalThis & { sdkworkDesktop?: ElectronBridgeShape }
  ).sdkworkDesktop;
}

function hostOk<T>(value: T): HostResult<T> {
  return { ok: true, value };
}

function fromError(error: unknown): HostResult<never> {
  const code =
    typeof error === 'object' && error !== null && 'code' in error
      ? ((error as { code: string }).code as never)
      : 'internal';
  const message = error instanceof Error ? error.message : String(error);
  return { ok: false, error: { code, message } };
}

export const ELECTRON_HOST_ID = 'electron' as const;

export interface ElectronDesktopHost {
  readonly id: typeof ELECTRON_HOST_ID;
  readonly capabilities: ReadonlySet<DesktopHostCapability>;
  hasCapability(capability: DesktopHostCapability): boolean;
  tray: TrayCapability;
  shortcuts: ShortcutCapability;
  clipboard: ClipboardCapability;
}

export function createElectronDesktopHost(): ElectronDesktopHost {
  const bridge = getBridge();
  const declared = bridge?.meta?.capabilities as DesktopHostCapability[] | undefined;
  const capabilities = new Set<DesktopHostCapability>(declared ?? []);

  return {
    id: ELECTRON_HOST_ID,
    capabilities,
    hasCapability(capability) {
      return capabilities.has(capability);
    },
    tray: {
      async setVisible(visible) {
        if (!bridge) return { ok: false, error: { code: 'unsupported', message: 'Electron host is unavailable.' } };
        try {
          await bridge.invoke(HOST_BRIDGE_METHODS.traySetVisible, { visible });
          return hostOk(undefined);
        } catch (error) {
          return fromError(error);
        }
      },
      async setMenu(request: TraySetMenuRequest) {
        if (!bridge) return { ok: false, error: { code: 'unsupported', message: 'Electron host is unavailable.' } };
        try {
          await bridge.invoke(HOST_BRIDGE_METHODS.traySetMenu, {
            items: request.items,
            tooltip: request.tooltip ?? null,
          });
          return hostOk(undefined);
        } catch (error) {
          return fromError(error);
        }
      },
      async restoreWindow() {
        if (!bridge) return { ok: false, error: { code: 'unsupported', message: 'Electron host is unavailable.' } };
        try {
          await bridge.invoke(HOST_BRIDGE_METHODS.trayRestoreWindow);
          return hostOk(undefined);
        } catch (error) {
          return fromError(error);
        }
      },
      onMenuActivated(listener) {
        if (!bridge) return () => {};
        return bridge.on(HOST_BRIDGE_METHODS.trayMenuActivated, (payload) =>
          listener(String(payload ?? '')),
        );
      },
    },
    shortcuts: {
      async registerAll(bindings: ShortcutBinding[]) {
        if (!bridge) return { ok: false, error: { code: 'unsupported', message: 'Electron host is unavailable.' } };
        try {
          const outcomes = await bridge.invoke<ShortcutRegistrationOutcome[]>(
            HOST_BRIDGE_METHODS.shortcutRegisterAll,
            { bindings },
          );
          return hostOk(outcomes);
        } catch (error) {
          return fromError(error);
        }
      },
      async unregister(binding: ShortcutBinding) {
        if (!bridge) return { ok: false, error: { code: 'unsupported', message: 'Electron host is unavailable.' } };
        try {
          await bridge.invoke(HOST_BRIDGE_METHODS.shortcutUnregister, { ...binding });
          return hostOk(undefined);
        } catch (error) {
          return fromError(error);
        }
      },
      async unregisterAll() {
        if (!bridge) return { ok: false, error: { code: 'unsupported', message: 'Electron host is unavailable.' } };
        try {
          await bridge.invoke(HOST_BRIDGE_METHODS.shortcutUnregisterAll);
          return hostOk(undefined);
        } catch (error) {
          return fromError(error);
        }
      },
      onTriggered(listener) {
        if (!bridge) return () => {};
        return bridge.on(HOST_BRIDGE_METHODS.shortcutTriggered, (payload) =>
          listener(String(payload ?? '')),
        );
      },
    },
    clipboard: {
      async cutPaths(request: ClipboardCutRequest) {
        if (!bridge) return { ok: false, error: { code: 'unsupported', message: 'Electron host is unavailable.' } };
        try {
          const result = await bridge.invoke<ClipboardCutResult>(
            HOST_BRIDGE_METHODS.clipboardCutPaths,
            { paths: request.paths },
          );
          return hostOk(result);
        } catch (error) {
          return fromError(error);
        }
      },
      async readPaths() {
        if (!bridge) return { ok: false, error: { code: 'unsupported', message: 'Electron host is unavailable.' } };
        try {
          const paths = await bridge.invoke<string[]>(HOST_BRIDGE_METHODS.clipboardReadPaths);
          return hostOk(paths);
        } catch (error) {
          return fromError(error);
        }
      },
      async writeText(text: string) {
        if (!bridge) return { ok: false, error: { code: 'unsupported', message: 'Electron host is unavailable.' } };
        try {
          await bridge.invoke(HOST_BRIDGE_METHODS.clipboardWriteText, { text });
          return hostOk(undefined);
        } catch (error) {
          return fromError(error);
        }
      },
    },
  };
}
