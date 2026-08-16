import type { SessionStorageLike } from './sessionStore';

type TauriInvoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

function resolveTauriInvoke(): TauriInvoke | undefined {
  if (typeof window === 'undefined') {
    return undefined;
  }

  const tauri = (globalThis as typeof globalThis & {
    __TAURI__?: { core?: { invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> } };
  }).__TAURI__;
  return tauri?.core?.invoke;
}

export function isOsSecureSessionStorageAvailable(): boolean {
  return resolveTauriInvoke() !== undefined;
}

export async function readOsSecureSessionSnapshot(): Promise<Record<string, string>> {
  const invoke = resolveTauriInvoke();
  if (!invoke) {
    return {};
  }

  try {
    return await invoke<Record<string, string>>('read_secure_session_snapshot');
  } catch {
    return {};
  }
}

export async function clearOsSecureSessionStorage(): Promise<void> {
  const invoke = resolveTauriInvoke();
  if (!invoke) {
    return;
  }

  try {
    await invoke('clear_secure_session_values');
  } catch {
    // Logout must remain resilient when secure storage cleanup fails.
  }
}

export async function createOsSecureSessionStorage(): Promise<SessionStorageLike | undefined> {
  if (typeof window === 'undefined') {
    return undefined;
  }

  const invoke = resolveTauriInvoke();
  if (!invoke) {
    return window.sessionStorage;
  }

  const memory = new Map<string, string>();
  const snapshot = await readOsSecureSessionSnapshot();
  for (const [key, value] of Object.entries(snapshot ?? {})) {
    memory.set(key, value);
  }

  return {
    getItem(key: string) {
      return memory.get(key) ?? null;
    },
    setItem(key: string, value: string) {
      memory.set(key, value);
      void invoke('write_secure_session_value', { request: { key, value } }).catch(() => {
        memory.delete(key);
      });
    },
    removeItem(key: string) {
      memory.delete(key);
      void invoke('remove_secure_session_value', { request: { key } }).catch(() => undefined);
    },
  };
}
