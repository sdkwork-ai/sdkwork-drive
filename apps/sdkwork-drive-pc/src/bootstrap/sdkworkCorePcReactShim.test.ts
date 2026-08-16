import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  clearPcReactRuntimeSession,
  persistPcReactRuntimeSession,
  readPcReactRuntimeSession,
} from './sdkworkCorePcReactShim';
import { DEFAULT_SESSION_STORAGE_KEY } from 'sdkwork-drive-pc-core';

class MemoryStorage {
  private readonly items = new Map<string, string>();

  getItem(key: string): string | null {
    return this.items.get(key) ?? null;
  }

  setItem(key: string, value: string): void {
    this.items.set(key, value);
  }

  removeItem(key: string): void {
    this.items.delete(key);
  }
}

function installBrowserStorage() {
  const originalWindow = globalThis.window;
  const localStorage = new MemoryStorage();
  const sessionStorage = new MemoryStorage();

  Object.defineProperty(globalThis, 'window', {
    configurable: true,
    value: {
      localStorage,
      sessionStorage,
    },
  });

  return {
    localStorage,
    restore: () => {
      Object.defineProperty(globalThis, 'window', {
        configurable: true,
        value: originalWindow,
      });
    },
    sessionStorage,
  };
}

describe('sdkworkCorePcReactShim', () => {
  afterEach(() => {
    vi.unstubAllEnvs();
  });

  it('does not read or write browser storage when desktop config requires secure storage', () => {
    const { localStorage, restore } = installBrowserStorage();
    vi.stubEnv('VITE_DRIVE_PC_RUNTIME_TARGET', 'desktop');
    localStorage.setItem(
      DEFAULT_SESSION_STORAGE_KEY,
      JSON.stringify({
        accessToken: 'stale-access-token',
        authToken: 'stale-auth-token',
      }),
    );

    try {
      expect(readPcReactRuntimeSession()).toEqual({});

      persistPcReactRuntimeSession({
        accessToken: 'next-access-token',
        authToken: 'next-auth-token',
      });
      clearPcReactRuntimeSession();

      expect(localStorage.getItem(DEFAULT_SESSION_STORAGE_KEY)).toContain('stale-access-token');
    } finally {
      restore();
    }
  });

  it('migrates legacy browser-session storage to durable local storage', () => {
    const { localStorage, restore, sessionStorage } = installBrowserStorage();
    vi.stubEnv('VITE_DRIVE_PC_TOKEN_STORAGE', 'browser-session');

    try {
      persistPcReactRuntimeSession({
        accessToken: 'Bearer access-token',
        authToken: 'Bearer auth-token',
        refreshToken: 'refresh-token',
      });

      expect(localStorage.getItem(DEFAULT_SESSION_STORAGE_KEY)).toContain('access-token');
      expect(sessionStorage.getItem(DEFAULT_SESSION_STORAGE_KEY)).toBeNull();
      expect(readPcReactRuntimeSession()).toEqual({
        accessToken: 'access-token',
        authToken: 'auth-token',
        refreshToken: 'refresh-token',
      });

      clearPcReactRuntimeSession();

      expect(localStorage.getItem(DEFAULT_SESSION_STORAGE_KEY)).toBeNull();
    } finally {
      restore();
    }
  });
});
