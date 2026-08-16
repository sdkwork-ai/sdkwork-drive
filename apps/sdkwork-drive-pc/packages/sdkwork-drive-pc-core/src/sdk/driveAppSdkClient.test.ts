import { describe, expect, it, vi } from 'vitest';
import { createRuntimeConfig } from '../config/runtimeConfig';
import {
  createSessionStore,
} from '../session/sessionStore';
import {
  createDriveSessionTokenManager,
} from '../session/sessionTokenManager';
import { createDriveAppSdkClient } from './driveAppSdkClient';
import type { DriveRuntimeConfig } from '../config/runtimeConfig';

const config: DriveRuntimeConfig = createRuntimeConfig({
  VITE_DRIVE_PC_ENVIRONMENT: 'test',
  VITE_DRIVE_PC_DEPLOYMENT_PROFILE: 'standalone',
  VITE_DRIVE_PC_DRIVE_APP_API_BASE_URL: 'https://drive.example.test',
  VITE_DRIVE_PC_DRIVE_ADMIN_STORAGE_API_BASE_URL:
    'https://drive-admin-storage.example.test',
});

describe('drive app sdk client', () => {
  it('binds the shared TokenManager and delegates requests through the generated Drive app SDK transport', async () => {
    const session = createSessionStore();
    session.setSession({
      authToken: 'auth-token',
      accessToken: 'access-token',
      context: {
        tenantId: 'tenant-001',
        userId: 'user-001',
        actorId: 'user-001',
        actorKind: 'user',
        permissionScope: ['drive.nodes.read', 'drive.nodes.write'],
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

    const client = createDriveAppSdkClient({
      config,
      sdkClient: sdkClient as never,
      tokenManager,
    });

    await client.request({
      operationId: 'nodes.list',
      pathParams: { spaceId: 'space-001' },
      query: { tenantId: 'tenant-001', page_size: 50, cursor: '100' },
    });

    expect(sdkClient.setTokenManager).toHaveBeenCalledWith(tokenManager);
    expect(tokenManager.getAuthToken()).toBe('auth-token');
    expect(tokenManager.getAccessToken()).toBe('access-token');
    expect(request).toHaveBeenCalledWith(
      '/app/v3/api/drive/spaces/space-001/nodes',
      {
        method: 'GET',
        params: { page_size: 50, cursor: '100' },
        body: undefined,
        contentType: undefined,
        signal: undefined,
      },
    );
    expect(request.mock.calls[0]![1]).not.toHaveProperty('headers');
  });

  it('normalizes generated SDK failures at the Drive app SDK facade boundary', async () => {
    const tokenManager = createDriveSessionTokenManager(createSessionStore());
    const request = vi.fn(async <T>(): Promise<T> => {
      throw Object.assign(new Error('tenantId is required'), {
        code: 40001,
        httpStatus: 400,
        traceId: 'trace-001',
      });
    });
    const sdkClient = {
      http: {
        request,
      },
      setTokenManager: vi.fn(),
    };

    const client = createDriveAppSdkClient({
      config,
      sdkClient: sdkClient as never,
      tokenManager,
    });

    await expect(client.request({
      operationId: 'nodes.list',
      pathParams: { spaceId: 'space-001' },
    })).rejects.toMatchObject({
      name: 'DriveAppSdkError',
      message: 'tenantId is required',
      operationId: 'nodes.list',
      status: 400,
      detail: 'tenantId is required',
      code: 40001,
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
    const client = createDriveAppSdkClient({
      config,
      sdkClient: sdkClient as never,
      tokenManager,
    });

    await expect(client.request({
      operationId: 'nodes.list',
      pathParams: { spaceId: 'space-001' },
    })).rejects.toBe(cancellation);
  });

  it('rejects legacy pagination query aliases at the SDK facade boundary', async () => {
    const tokenManager = createDriveSessionTokenManager(createSessionStore());
    const request = vi.fn(async <T>(): Promise<T> => ({ items: [] }) as T);
    const sdkClient = {
      http: {
        request,
      },
      setTokenManager: vi.fn(),
    };

    const client = createDriveAppSdkClient({
      config,
      sdkClient: sdkClient as never,
      tokenManager,
    });

    await expect(client.request({
      operationId: 'nodes.list',
      pathParams: { spaceId: 'space-001' },
      query: { pageSize: 50 },
    })).rejects.toThrow(/Legacy SDKWork pagination query parameter "pageSize"/);
    expect(request).not.toHaveBeenCalled();
  });
});
