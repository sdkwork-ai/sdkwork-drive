import { describe, expect, it, vi } from 'vitest';
import {
  createDriveSessionTokenManager,
  createRuntimeConfig,
  createSessionStore,
  type DriveRuntimeConfig,
} from 'sdkwork-drive-pc-core';
import { createDriveAdminStorageSdkClient } from './driveAdminStorageSdkClient';

const config: DriveRuntimeConfig = createRuntimeConfig({
  VITE_DRIVE_PC_ENVIRONMENT: 'test',
  VITE_DRIVE_PC_DEPLOYMENT_PROFILE: 'standalone',
  VITE_DRIVE_PC_DRIVE_APP_API_BASE_URL: 'https://drive.example.test',
  VITE_DRIVE_PC_DRIVE_ADMIN_STORAGE_API_BASE_URL:
    'https://drive-admin-storage.example.test',
});

describe('drive admin storage sdk client', () => {
  it('binds the shared TokenManager and delegates requests through the generated admin storage SDK transport', async () => {
    const session = createSessionStore();
    session.setSession({
      authToken: 'auth-token',
      accessToken: 'access-token',
      context: {
        tenantId: 'tenant-001',
        userId: 'user-001',
        actorId: 'user-001',
        actorKind: 'user',
        permissionScope: ['drive.storage.admin'],
        dataScope: ['tenant'],
      },
    });
    const tokenManager = createDriveSessionTokenManager(session);
    const request = vi.fn(async <T>(
      _path: string,
      _options?: Record<string, unknown>,
    ): Promise<T> => ({ items: [] }) as T);
    const sdkClient = {
      http: {
        request,
      },
      setTokenManager: vi.fn(),
    };

    const client = createDriveAdminStorageSdkClient({
      config,
      sdkClient: sdkClient as never,
      tokenManager,
    });

    await client.request({
      operationId: 'storageProviders.list',
      query: { status: 'active', page_size: 50, cursor: '100' },
    });

    expect(sdkClient.setTokenManager).toHaveBeenCalledWith(tokenManager);
    expect(tokenManager.getAuthToken()).toBe('auth-token');
    expect(tokenManager.getAccessToken()).toBe('access-token');
    expect(request).toHaveBeenCalledWith(
      '/backend/v3/api/drive/storage/providers',
      {
        method: 'GET',
        params: { status: 'active', page_size: 50, cursor: '100' },
        body: undefined,
        contentType: undefined,
        signal: undefined,
      },
    );
    expect(request.mock.calls[0]![1]).not.toHaveProperty('headers');
  });

  it('normalizes generated SDK failures at the admin storage SDK facade boundary', async () => {
    const tokenManager = createDriveSessionTokenManager(createSessionStore());
    const request = vi.fn(async <T>(): Promise<T> => {
      throw Object.assign(new Error('admin storage permission is required'), {
        code: 40301,
        httpStatus: 403,
        traceId: 'trace-001',
      });
    });
    const sdkClient = {
      http: {
        request,
      },
      setTokenManager: vi.fn(),
    };

    const client = createDriveAdminStorageSdkClient({
      config,
      sdkClient: sdkClient as never,
      tokenManager,
    });

    await expect(client.request({
      operationId: 'storageProviders.retrieve',
      pathParams: { providerId: 'provider-s3-primary' },
    })).rejects.toMatchObject({
      name: 'DriveAdminStorageSdkError',
      message: 'admin storage permission is required',
      operationId: 'storageProviders.retrieve',
      status: 403,
      detail: 'admin storage permission is required',
      code: 40301,
      traceId: 'trace-001',
    });
  });

  it('preserves generated SDK cancellation errors for effect cleanup', async () => {
    const tokenManager = createDriveSessionTokenManager(createSessionStore());
    const cancellation = Object.assign(new Error('Request was cancelled'), {
      code: 'CANCELLED',
      name: 'CancelledError',
    });
    const sdkClient = {
      http: {
        request: vi.fn(async () => {
          throw cancellation;
        }),
      },
      setTokenManager: vi.fn(),
    };
    const client = createDriveAdminStorageSdkClient({
      config,
      sdkClient: sdkClient as never,
      tokenManager,
    });

    await expect(client.request({ operationId: 'storageProviders.list' })).rejects.toBe(cancellation);
  });
});
