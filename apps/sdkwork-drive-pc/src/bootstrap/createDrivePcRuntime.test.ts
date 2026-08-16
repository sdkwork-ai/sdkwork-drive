import { afterEach, describe, expect, it, vi } from 'vitest';
import { DEFAULT_SESSION_STORAGE_KEY, hasDriveIamSession } from 'sdkwork-drive-pc-core';
import { createDrivePcRuntime } from './createDrivePcRuntime';

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

describe('createDrivePcRuntime', () => {
  afterEach(() => {
    vi.unstubAllEnvs();
  });

  it('does not seed an IAM session before appbase login completes', async () => {
    const originalWindow = globalThis.window;
    const localStorage = new MemoryStorage();
    Object.defineProperty(globalThis, 'window', {
      configurable: true,
      value: { localStorage },
    });

    try {
      const runtime = await createDrivePcRuntime();

      expect(hasDriveIamSession(runtime.session.getSnapshot())).toBe(false);
      expect(runtime.session.getSnapshot().context).toBeUndefined();
    } finally {
      Object.defineProperty(globalThis, 'window', {
        configurable: true,
        value: originalWindow,
      });
    }
  });

  it('defaults local PC runtime to the Drive App SDK data service without a prebuilt IAM context', async () => {
    const runtime = await createDrivePcRuntime();
    const session = runtime.session.getSnapshot();

    expect(hasDriveIamSession(session)).toBe(false);
    expect(session.context).toBeUndefined();
  });

  it('keeps a persisted real IAM session for the generated App SDK wrapper', async () => {
    const originalWindow = globalThis.window;
    const localStorage = new MemoryStorage();
    vi.stubEnv('VITE_DRIVE_PC_TOKEN_STORAGE', 'browser-local');
    localStorage.setItem(
      DEFAULT_SESSION_STORAGE_KEY,
      JSON.stringify({
        authToken: 'auth-token',
        accessToken: 'access-token',
        refreshToken: 'refresh-token',
        sessionId: 'session-001',
        user: {
          id: 'user-001',
        },
        context: {
          tenantId: 'tenant-001',
          userId: 'user-001',
          deploymentMode: 'saas',
          authLevel: 'password',
        },
      }),
    );
    Object.defineProperty(globalThis, 'window', {
      configurable: true,
      value: { localStorage },
    });

    try {
      const runtime = await createDrivePcRuntime();

      expect(hasDriveIamSession(runtime.session.getSnapshot())).toBe(true);
      expect(runtime.session.getSnapshot()).toMatchObject({
        authToken: 'auth-token',
        accessToken: 'access-token',
        context: {
          tenantId: 'tenant-001',
          userId: 'user-001',
        },
      });
    } finally {
      Object.defineProperty(globalThis, 'window', {
        configurable: true,
        value: originalWindow,
      });
    }
  });

  it('keeps dependency SDK base URLs explicit for appbase, Drive app, and Drive admin storage', async () => {
    vi.stubEnv('VITE_DRIVE_PC_APPBASE_APP_API_BASE_URL', 'https://appbase.example.test/app/v3/api');
    vi.stubEnv('VITE_DRIVE_PC_DRIVE_APP_API_BASE_URL', 'https://drive-app.example.test/app/v3/api');
    vi.stubEnv(
      'VITE_DRIVE_PC_DRIVE_ADMIN_STORAGE_API_BASE_URL',
      'https://drive-admin-storage.example.test/backend/v3/api',
    );

    const runtime = await createDrivePcRuntime();

    expect(runtime.config.sdkBaseUrls.dependencySdkBaseUrls).toMatchObject({
      'sdkwork-iam-app-sdk': {
        appApiBaseUrl: 'https://appbase.example.test/app/v3/api',
      },
      'sdkwork-drive-app-sdk': {
        appApiBaseUrl: 'https://drive-app.example.test/app/v3/api',
      },
      'sdkwork-drive-admin-storage-sdk': {
        backendApiBaseUrl: 'https://drive-admin-storage.example.test/backend/v3/api',
      },
    });
  });

  it('exposes admin storage SDK outside the app SDK runtime surface', async () => {
    const runtime = await createDrivePcRuntime();

    expect(runtime.sdk.app).toBeDefined();
    expect(runtime.admin.adminStorage).toBeDefined();
    expect(runtime.admin.backend).toBeDefined();
    expect('adminStorage' in runtime.sdk).toBe(false);
  });

  it('does not fall back to browser localStorage when desktop config requires secure storage', async () => {
    const originalWindow = globalThis.window;
    const localStorage = new MemoryStorage();
    vi.stubEnv('VITE_DRIVE_PC_RUNTIME_TARGET', 'desktop');
    localStorage.setItem(
      DEFAULT_SESSION_STORAGE_KEY,
      JSON.stringify({
        authToken: 'auth-token',
        accessToken: 'access-token',
        refreshToken: 'refresh-token',
        context: {
          tenantId: 'tenant-001',
          userId: 'user-001',
        },
      }),
    );
    Object.defineProperty(globalThis, 'window', {
      configurable: true,
      value: { localStorage },
    });

    try {
      const runtime = await createDrivePcRuntime();

      expect(runtime.config.auth.tokenStorage).toBe('os-secure-storage');
      expect(hasDriveIamSession(runtime.session.getSnapshot())).toBe(false);
      expect(localStorage.getItem(DEFAULT_SESSION_STORAGE_KEY)).toContain('auth-token');
    } finally {
      Object.defineProperty(globalThis, 'window', {
        configurable: true,
        value: originalWindow,
      });
    }
  });
});
