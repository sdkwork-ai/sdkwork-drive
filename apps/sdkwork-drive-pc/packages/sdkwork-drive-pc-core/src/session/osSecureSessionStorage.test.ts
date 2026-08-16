import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  clearOsSecureSessionStorage,
  createOsSecureSessionStorage,
  isOsSecureSessionStorageAvailable,
  readOsSecureSessionSnapshot,
} from './osSecureSessionStorage';

describe('osSecureSessionStorage', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.unstubAllEnvs();
  });

  it('reports unavailable storage when Tauri invoke is missing', () => {
    expect(isOsSecureSessionStorageAvailable()).toBe(false);
  });

  it('hydrates secure storage before returning a readable snapshot', async () => {
    const invoke = vi.fn(async (command: string) => {
      if (command === 'read_secure_session_snapshot') {
        return { 'sdkwork-drive-pc-session': '{"authToken":"secure-auth"}' };
      }
      return undefined;
    });
    vi.stubGlobal('window', {
      sessionStorage: {
        getItem: vi.fn(),
        setItem: vi.fn(),
        removeItem: vi.fn(),
      },
    });
    (globalThis as typeof globalThis & { __TAURI__?: unknown }).__TAURI__ = {
      core: { invoke },
    };

    const storage = await createOsSecureSessionStorage();

    expect(storage?.getItem('sdkwork-drive-pc-session')).toBe('{"authToken":"secure-auth"}');
    expect(invoke).toHaveBeenCalledWith('read_secure_session_snapshot');
  });

  it('clears secure storage through the Tauri command', async () => {
    const invoke = vi.fn(async () => undefined);
    vi.stubGlobal('window', {});
    (globalThis as typeof globalThis & { __TAURI__?: unknown }).__TAURI__ = {
      core: { invoke },
    };

    await clearOsSecureSessionStorage();

    expect(invoke).toHaveBeenCalledWith('clear_secure_session_values');
  });

  it('returns an empty snapshot when secure storage read fails', async () => {
    const invoke = vi.fn(async () => {
      throw new Error('keyring unavailable');
    });
    (globalThis as typeof globalThis & { __TAURI__?: unknown }).__TAURI__ = {
      core: { invoke },
    };

    await expect(readOsSecureSessionSnapshot()).resolves.toEqual({});
  });
});
