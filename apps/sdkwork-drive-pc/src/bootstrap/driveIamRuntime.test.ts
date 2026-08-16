import { describe, expect, it, vi } from 'vitest';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import {
  createDriveSessionTokenManager,
  createRuntimeConfig,
  createSessionStore,
} from 'sdkwork-drive-pc-core';
import { createDriveIamRuntime } from './driveIamRuntime';
import type { DriveRuntimeConfig } from 'sdkwork-drive-pc-core';

const config: DriveRuntimeConfig = createRuntimeConfig({
  VITE_DRIVE_PC_ENVIRONMENT: 'test',
  VITE_DRIVE_PC_DEPLOYMENT_PROFILE: 'standalone',
  VITE_DRIVE_PC_DRIVE_APP_API_BASE_URL: 'https://drive.example.test',
  VITE_DRIVE_PC_DRIVE_ADMIN_STORAGE_API_BASE_URL:
    'https://drive-admin-storage.example.test',
});

function createTestJwt(payload: Record<string, unknown>): string {
  const header = Buffer.from(JSON.stringify({ alg: 'none', typ: 'JWT' })).toString('base64url');
  const body = Buffer.from(JSON.stringify(payload)).toString('base64url');
  return `${header}.${body}.signature`;
}

const TEST_AUTH_TOKEN = createTestJwt({ sub: 'user-001', typ: 'auth', token_version: 1 });
const TEST_ACCESS_TOKEN = createTestJwt({ sub: 'user-001', typ: 'access', token_version: 1 });
const TEST_REFRESH_TOKEN = 'refresh-token';

function createOAuthClient() {
  return {
    authorizations: {
      completions: { create: vi.fn() },
    },
    deviceAuthorizations: {
      create: vi.fn(),
      retrieve: vi.fn(),
      passwordCompletions: { create: vi.fn() },
      scans: { create: vi.fn() },
      sessionExchanges: { create: vi.fn() },
    },
    accountLinks: {
      delete: vi.fn(),
      list: vi.fn(),
    },
    authorizationUrls: { create: vi.fn() },
    callbacks: {
      handleGet: vi.fn(),
      handlePost: vi.fn(),
    },
    grants: {
      delete: vi.fn(),
      list: vi.fn(),
    },
    miniProgramSessions: { create: vi.fn() },
    providers: { list: vi.fn() },
    sessions: { create: vi.fn() },
  };
}

function createIamDirectoryClient() {
  return {
    departmentAssignments: { list: vi.fn() },
    departments: {
      list: vi.fn(),
      tree: { retrieve: vi.fn() },
    },
    organizationMemberships: { list: vi.fn() },
    organizations: {
      list: vi.fn(),
      tree: { retrieve: vi.fn() },
    },
    positionAssignments: { list: vi.fn() },
    positions: { list: vi.fn() },
    roleBindings: { list: vi.fn() },
    users: {
      current: {
        retrieve: vi.fn(),
        update: vi.fn(),
        emailBindings: {
          create: vi.fn(),
          delete: vi.fn(),
        },
        phoneBindings: {
          create: vi.fn(),
          delete: vi.fn(),
        },
        password: {
          update: vi.fn(),
        },
      },
    },
  };
}

describe('drive IAM runtime bridge', () => {
  it('uses the high-level appbase PC auth runtime instead of product-local low-level IAM wiring', () => {
    const source = readFileSync(
      path.join(process.cwd(), 'src/bootstrap/driveIamRuntime.ts'),
      'utf8',
    );

    expect(source).toContain('@sdkwork/auth-runtime-pc-react');
    expect(source).toContain('createSdkworkAppbasePcAuthRuntime');
    expect(source).not.toMatch(/@sdkwork\/iam-sdk-adapter|createIamSdkAdapters/u);
    expect(source).not.toMatch(/\bcreateIamRuntime\(/u);
  });

  it('creates an appbase IAM runtime backed by the generated appbase app SDK surface', async () => {
    const session = createSessionStore();
    const runtime = createDriveIamRuntime({ config, session });

    expect(runtime.config).toMatchObject({
      appId: 'sdkwork-drive-pc',
      deploymentMode: 'private',
      environment: 'test',
    });
    expect(runtime.service.auth.sessions.create).toEqual(expect.any(Function));
    expect(runtime.service.auth.sessions.current.retrieve).toEqual(expect.any(Function));
    expect(runtime.service.auth.sessions.refresh).toEqual(expect.any(Function));
    expect(runtime.service.system.iam.runtime.retrieve).toEqual(expect.any(Function));
    expect(runtime.service.system.iam.verificationPolicy.retrieve).toEqual(expect.any(Function));
    expect(runtime.service.iam.users.current.retrieve).toEqual(expect.any(Function));

    await runtime.tokenStore.set({
      authToken: 'auth-token',
      accessToken: 'access-token',
      refreshToken: 'refresh-token',
    });

    await expect(runtime.getAuthHeaders()).resolves.toMatchObject({
      Authorization: 'Bearer auth-token',
      'Access-Token': 'access-token',
    });
  });

  it('binds the same global TokenManager to app SDK clients registered with IAM runtime', () => {
    const session = createSessionStore();
    const tokenManager = createDriveSessionTokenManager(session);
    const appSdkClient = { setTokenManager: vi.fn() };

    const runtime = createDriveIamRuntime({
      config,
      sdkClients: [appSdkClient],
      session,
      tokenManager,
    });

    expect(runtime.tokenManager).toBe(tokenManager);
    expect(appSdkClient.setTokenManager).toHaveBeenCalledWith(tokenManager);
  });

  it('creates appbase IAM SDK clients with the configured appbase app-api base URL', async () => {
    const session = createSessionStore();
    const runtime = createDriveIamRuntime({
      config: createRuntimeConfig({
        VITE_DRIVE_PC_ENVIRONMENT: 'test',
        VITE_DRIVE_PC_DEPLOYMENT_PROFILE: 'standalone',
        VITE_DRIVE_PC_DRIVE_APP_API_BASE_URL: 'https://drive.example.test',
        VITE_DRIVE_PC_APPBASE_APP_API_BASE_URL: 'https://appbase.example.test/app/v3/api',
      }),
      session,
    });
    const fetchMock = vi.fn(async () =>
      new Response(
        JSON.stringify({
          code: 0,
          data: {
            appId: 'sdkwork-drive-pc',
          },
        }),
        {
          headers: {
            'Content-Type': 'application/json',
          },
          status: 200,
        },
      ),
    );
    vi.stubGlobal('fetch', fetchMock);

    await runtime.service.system.iam.runtime.retrieve();

    expect(fetchMock).toHaveBeenCalledWith(
      'https://appbase.example.test/app/v3/api/system/iam/runtime',
      expect.objectContaining({
        method: 'GET',
      }),
    );
  });

  it('persists appbase IAM session tokens, user identity, and app context into the Drive session store', async () => {
    const session = createSessionStore();
    const runtime = createDriveIamRuntime({ config, session });

    await runtime.tokenStore.set({
      authToken: 'auth-token',
      accessToken: 'access-token',
      refreshToken: 'refresh-token',
    });
    await runtime.contextStore.setAppContext({
      appId: 'sdkwork-drive-pc',
      authLevel: 'password',
      dataScope: ['tenant'],
      deploymentMode: 'private',
      environment: 'test',
      organizationId: 'org-001',
      permissionScope: ['drive.nodes.read'],
      sessionId: 'session-001',
      tenantId: 'tenant-001',
      userId: 'user-001',
    });
    runtime.onCurrentUserChanged?.({
      id: 'user-001',
      displayName: 'Ada Lovelace',
      email: 'ada@sdkwork.local',
    });

    expect(session.getSnapshot()).toMatchObject({
      authToken: 'auth-token',
      accessToken: 'access-token',
      refreshToken: 'refresh-token',
      sessionId: 'session-001',
      user: {
        id: 'user-001',
        displayName: 'Ada Lovelace',
        email: 'ada@sdkwork.local',
      },
      context: {
        tenantId: 'tenant-001',
        userId: 'user-001',
        organizationId: 'org-001',
        sessionId: 'session-001',
        appId: 'sdkwork-drive-pc',
        environment: 'test',
        deploymentMode: 'private',
        authLevel: 'password',
        dataScope: ['tenant'],
        permissionScope: ['drive.nodes.read'],
        actorId: 'user-001',
        actorKind: 'user',
      },
    });
  });

  it('clears Drive session state after appbase IAM logout callbacks clear tokens and context', async () => {
    const session = createSessionStore();
    const runtime = createDriveIamRuntime({ config, session });
    await runtime.tokenStore.set({
      authToken: 'auth-token',
      accessToken: 'access-token',
    });

    await runtime.tokenStore.clear();

    expect(session.getSnapshot()).toEqual({});
  });

  it('lets tests inject a generated app SDK client while preserving the IAM adapter contract', async () => {
    const session = createSessionStore();
    const sessionsCreate = vi.fn(async () => ({
      accessToken: TEST_ACCESS_TOKEN,
      authToken: TEST_AUTH_TOKEN,
      refreshToken: TEST_REFRESH_TOKEN,
      context: {
        appId: 'sdkwork-drive-pc',
        authLevel: 'password',
        dataScope: ['tenant'],
        deploymentMode: 'private',
        environment: 'test',
        permissionScope: ['drive.*'],
        sessionId: 'session-001',
        tenantId: 'tenant-001',
        userId: 'user-001',
      },
      user: {
        id: 'user-001',
        displayName: 'Ada Lovelace',
      },
    }));

    const runtime = createDriveIamRuntime({
      config,
      session,
      appClient: {
        auth: {
          passwordResetRequests: { create: vi.fn() },
          passwordResets: { create: vi.fn() },
          registrations: { create: vi.fn() },
          sessions: {
            create: sessionsCreate,
            current: {
              delete: vi.fn(),
              retrieve: vi.fn(),
              update: vi.fn(),
            },
            refresh: vi.fn(),
            organizationSelection: { create: vi.fn() },
            tenantSelection: { create: vi.fn() },
            loginContextSelection: { create: vi.fn() },
          },
        },
        iam: createIamDirectoryClient(),
        oauth: createOAuthClient(),
        system: {
          iam: {
            runtime: { retrieve: vi.fn() },
            verificationPolicy: { retrieve: vi.fn() },
            accountBindingPolicy: { retrieve: vi.fn() },
          },
        },
      },
    });

    await runtime.service.auth.sessions.create({
      grantType: 'password',
      username: 'ada',
      password: 'secret',
    });

    expect(sessionsCreate).toHaveBeenCalledWith({
      grantType: 'password',
      username: 'ada',
      password: 'secret',
    });
    expect(session.getSnapshot()).toMatchObject({
      authToken: TEST_AUTH_TOKEN,
      accessToken: TEST_ACCESS_TOKEN,
      context: {
        tenantId: 'tenant-001',
        userId: 'user-001',
      },
      user: {
        id: 'user-001',
        displayName: 'Ada Lovelace',
      },
    });
  });

  it('maps organizationSelection to tenantSelection for IAM SDK port compatibility', () => {
    const source = readFileSync(
      path.join(process.cwd(), 'src/bootstrap/driveIamRuntime.ts'),
      'utf8',
    );

    expect(source).toContain('ensureIamTenantSelectionCompat');
    expect(source).toContain('organizationSelection.create.bind');
    expect(source).toContain('sessions.tenantSelection');
  });

  it('hydrates Drive AppContext from current session when login only returns tokens', async () => {
    const session = createSessionStore();
    const sessionsCurrentRetrieve = vi.fn(async () => ({
      accessToken: TEST_ACCESS_TOKEN,
      authToken: TEST_AUTH_TOKEN,
      context: {
        appId: 'sdkwork-drive-pc',
        authLevel: 'password',
        dataScope: ['tenant'],
        deploymentMode: 'private',
        environment: 'test',
        permissionScope: ['drive.*'],
        sessionId: 'session-001',
        tenantId: 'tenant-001',
        userId: 'user-001',
      },
    }));
    const runtime = createDriveIamRuntime({
      config,
      session,
      appClient: {
        auth: {
          passwordResetRequests: { create: vi.fn() },
          passwordResets: { create: vi.fn() },
          registrations: { create: vi.fn() },
          sessions: {
            create: vi.fn(async () => ({
              accessToken: TEST_ACCESS_TOKEN,
              authToken: TEST_AUTH_TOKEN,
              refreshToken: TEST_REFRESH_TOKEN,
            })),
            current: {
              delete: vi.fn(),
              retrieve: sessionsCurrentRetrieve,
              update: vi.fn(),
            },
            refresh: vi.fn(),
            organizationSelection: { create: vi.fn() },
            tenantSelection: { create: vi.fn() },
            loginContextSelection: { create: vi.fn() },
          },
        },
        iam: createIamDirectoryClient(),
        oauth: createOAuthClient(),
        system: {
          iam: {
            runtime: { retrieve: vi.fn() },
            verificationPolicy: { retrieve: vi.fn() },
            accountBindingPolicy: { retrieve: vi.fn() },
          },
        },
      },
    });

    await runtime.service.auth.sessions.create({
      grantType: 'password',
      username: 'ada',
      password: 'secret',
    });

    expect(sessionsCurrentRetrieve).toHaveBeenCalledTimes(1);
    expect(session.getSnapshot().context).toMatchObject({
      tenantId: 'tenant-001',
      userId: 'user-001',
      sessionId: 'session-001',
    });
  });
});
