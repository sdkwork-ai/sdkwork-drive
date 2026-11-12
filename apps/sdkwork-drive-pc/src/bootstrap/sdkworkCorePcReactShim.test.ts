import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  clearPcReactRuntimeSession,
  persistPcReactRuntimeSession,
  readPcReactRuntimeSession,
} from './sdkworkCorePcReactShim';
import { DEFAULT_SESSION_STORAGE_KEY } from 'sdkwork-drive-pc-core';

class MemoryStorage {
  private readonly items = new Map<string, string>();

  get length(): number {
    return this.items.size;
  }

  key(index: number): string | null {
    return [...this.items.keys()][index] ?? null;
  }

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

  it('persists browser-session tokens into session storage only', () => {
    const { localStorage, restore, sessionStorage } = installBrowserStorage();
    vi.stubEnv('VITE_DRIVE_PC_TOKEN_STORAGE', 'browser-session');

    try {
      persistPcReactRuntimeSession({
        accessToken: 'Bearer access-token',
        authToken: 'Bearer auth-token',
        refreshToken: 'refresh-token',
      });

      // A durable copy would keep the previous account's credentials on a shared
      // device after the browser session ends.
      expect(localStorage.getItem(DEFAULT_SESSION_STORAGE_KEY)).toBeNull();
      expect(sessionStorage.getItem(DEFAULT_SESSION_STORAGE_KEY)).toContain('access-token');
      expect(readPcReactRuntimeSession()).toMatchObject({
        accessToken: 'access-token',
        authToken: 'auth-token',
        refreshToken: 'refresh-token',
      });

      clearPcReactRuntimeSession();

      expect(sessionStorage.getItem(DEFAULT_SESSION_STORAGE_KEY)).toBeNull();
    } finally {
      restore();
    }
  });

  it('adopts a durable legacy session once for a browser-session runtime', () => {
    const { localStorage, restore, sessionStorage } = installBrowserStorage();
    vi.stubEnv('VITE_DRIVE_PC_TOKEN_STORAGE', 'browser-session');
    localStorage.setItem(
      DEFAULT_SESSION_STORAGE_KEY,
      JSON.stringify({
        accessToken: 'legacy-access-token',
        authToken: 'legacy-auth-token',
        context: { tenantId: 'tenant-001', userId: 'user-001' },
      }),
    );

    try {
      expect(readPcReactRuntimeSession()).toMatchObject({
        accessToken: 'legacy-access-token',
        context: { tenantId: 'tenant-001', userId: 'user-001' },
      });
      expect(sessionStorage.getItem(DEFAULT_SESSION_STORAGE_KEY)).toContain('legacy-access-token');
      expect(localStorage.getItem(DEFAULT_SESSION_STORAGE_KEY)).toBeNull();
    } finally {
      restore();
    }
  });

  it('keeps the tenant and user context when the auth flow commits tokens only', () => {
    const { localStorage, restore } = installBrowserStorage();
    vi.stubEnv('VITE_DRIVE_PC_TOKEN_STORAGE', 'browser-local');
    localStorage.setItem(
      DEFAULT_SESSION_STORAGE_KEY,
      JSON.stringify({
        accessToken: 'old-access-token',
        authToken: 'auth-token',
        context: { tenantId: 'tenant-001', userId: 'user-001' },
        user: { id: 'user-001', displayName: 'Ada Lovelace' },
      }),
    );

    try {
      persistPcReactRuntimeSession({ accessToken: 'rotated-access-token' });

      const stored = JSON.parse(localStorage.getItem(DEFAULT_SESSION_STORAGE_KEY) ?? '{}');
      expect(stored).toMatchObject({
        accessToken: 'rotated-access-token',
        authToken: 'auth-token',
        context: { tenantId: 'tenant-001', userId: 'user-001' },
        user: { id: 'user-001' },
      });
      expect(readPcReactRuntimeSession().context).toEqual({
        tenantId: 'tenant-001',
        userId: 'user-001',
      });
    } finally {
      restore();
    }
  });

  it('replaces the previous account context when a new one is committed', () => {
    const { restore } = installBrowserStorage();
    vi.stubEnv('VITE_DRIVE_PC_TOKEN_STORAGE', 'browser-local');

    try {
      persistPcReactRuntimeSession({
        accessToken: 'access-token-a',
        authToken: 'auth-token-a',
        context: { tenantId: 'tenant-001', userId: 'user-a' },
        user: { id: 'user-a' },
      });

      persistPcReactRuntimeSession({
        accessToken: 'access-token-b',
        authToken: 'auth-token-b',
        context: { tenantId: 'tenant-002', userId: 'user-b' },
        user: { id: 'user-b' },
      });

      expect(readPcReactRuntimeSession()).toMatchObject({
        accessToken: 'access-token-b',
        context: { tenantId: 'tenant-002', userId: 'user-b' },
        user: { id: 'user-b' },
      });
    } finally {
      restore();
    }
  });

  it('drops the context with the tokens when the session is cleared', () => {
    const { localStorage, restore } = installBrowserStorage();
    vi.stubEnv('VITE_DRIVE_PC_TOKEN_STORAGE', 'browser-local');

    try {
      persistPcReactRuntimeSession({
        accessToken: 'access-token',
        authToken: 'auth-token',
        context: { tenantId: 'tenant-001', userId: 'user-001' },
      });

      persistPcReactRuntimeSession({ accessToken: '', authToken: '', refreshToken: '' });

      expect(localStorage.getItem(DEFAULT_SESSION_STORAGE_KEY)).toBeNull();
      expect(readPcReactRuntimeSession()).toEqual({});
    } finally {
      restore();
    }
  });
});
