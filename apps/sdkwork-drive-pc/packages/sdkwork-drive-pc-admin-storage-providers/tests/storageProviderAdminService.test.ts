import { describe, expect, it } from 'vitest';
import type {
  DriveAdminStorageSdkClient,
  DriveAdminStorageSdkRequest,
} from 'sdkwork-drive-pc-admin-core';
import { DriveAdminStorageSdkError } from 'sdkwork-drive-pc-admin-core';
import {
  createStorageProviderAdminService,
  type StorageProviderAdminService,
} from '../src/services/storageProviderAdminService';

function createFakeService() {
  const calls: DriveAdminStorageSdkRequest[] = [];
  const client = {
    metadata: {},
    operations: {},
    setTokenManager: () => undefined,
    async request<T>(request: DriveAdminStorageSdkRequest): Promise<T> {
      calls.push(request);
      return responseFor(request) as T;
    },
  } as unknown as DriveAdminStorageSdkClient;

  const service = createStorageProviderAdminService({
    adminStorageSdkClient: client,
    getSession: () => ({
      context: {
        tenantId: 'tenant-100',
        userId: 'user-100',
        actorId: 'operator-100',
      },
    }),
  });

  return { calls, service };
}

function responseFor(request: DriveAdminStorageSdkRequest): unknown {
  if (request.operationId === 'storageProviderKinds.list' || request.operationId === 'storageProviderKinds.initialize') {
    return {
      items: [
        {
          providerKind: 'aliyun_oss',
          displayName: 'Alibaba Cloud OSS',
          enabled: true,
          sortOrder: 4,
          version: 1,
          configCount: 2,
        },
        {
          providerKind: 'tencent_cos',
          displayName: 'Tencent Cloud COS',
          enabled: false,
          sortOrder: 5,
          version: 2,
          configCount: 0,
        },
      ],
    };
  }

  if (request.operationId === 'storageProviderKinds.update') {
    return {
      providerKind: request.pathParams?.providerKind ?? 'aliyun_oss',
      displayName: 'Alibaba Cloud OSS',
      enabled: (request.body as { enabled?: boolean } | undefined)?.enabled === true,
      sortOrder: 4,
      version: 2,
      configCount: 2,
    };
  }

  if (request.operationId === 'storageProviders.list') {
    return {
      items: [
        {
          id: 'provider-cos',
          providerKind: 'tencent_cos',
          name: 'Tencent COS',
          endpointUrl: 'https://cos.ap-shanghai.myqcloud.com',
          region: 'ap-shanghai',
          bucket: 'drive-prod',
          pathStyle: false,
          credentialRef: 'secret/tencent-cos',
          status: 'active',
          version: 2,
          credentialConfigured: true,
        },
      ],
    };
  }

  if (request.operationId === 'storageProviders.create' || request.operationId === 'storageProviders.update') {
    return {
      id: request.pathParams?.providerId ?? 'provider-s3',
      providerKind: 's3_compatible',
      name: 'Amazon S3',
      endpointUrl: 'https://s3.us-east-1.amazonaws.com',
      region: 'us-east-1',
      bucket: 'drive-prod',
      pathStyle: false,
      status: 'active',
      version: 1,
      credentialConfigured: true,
    };
  }

  if (request.operationId === 'storageProviderBindings.default.update') {
    return {
      id: 'binding-default',
      providerId: 'provider-s3',
      bindingScope: 'tenant',
      purpose: 'default',
      lifecycleStatus: 'active',
      version: 1,
      storageProvider: {
        id: 'provider-s3',
        providerKind: 's3_compatible',
        name: 'Amazon S3',
        endpointUrl: 'https://s3.us-east-1.amazonaws.com',
        bucket: 'drive-prod',
        pathStyle: false,
        status: 'active',
        version: 1,
        credentialConfigured: true,
      },
    };
  }

  if (request.operationId === 'storageProviders.test') {
    return { reachable: true };
  }

  if (request.operationId === 'storageProviders.objects.list') {
    return {
      providerId: request.pathParams?.providerId ?? 'provider-s3',
      bucket: 'drive-prod',
      items: [
        {
          providerId: request.pathParams?.providerId ?? 'provider-s3',
          bucket: 'drive-prod',
          objectKind: 'prefix',
          objectKey: 'docs/',
          contentLength: 0,
        },
        {
          providerId: request.pathParams?.providerId ?? 'provider-s3',
          bucket: 'drive-prod',
          objectKind: 'object',
          objectKey: 'docs/readme.txt',
          contentLength: 2048,
          contentType: 'text/plain',
          lastModifiedEpochMs: 1700000000000,
        },
      ],
      pageInfo: { mode: 'cursor', hasMore: false },
    };
  }

  if (request.operationId === 'storageProviders.delete') {
    return { deleted: true };
  }

  if (request.operationId === 'storageProviders.objects.delete') {
    return { deleted: true };
  }

  if (request.operationId === 'storageProviders.objects.content.retrieve') {
    return {
      providerId: request.pathParams?.providerId ?? 'provider-s3',
      bucket: 'drive-prod',
      objectKey: request.pathParams?.objectKey ?? 'docs/readme.txt',
      contentType: 'text/plain',
      sizeBytes: 5,
      encoding: 'base64',
      content: 'aGVsbG8=',
      checksumSha256: '2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824',
    };
  }

  if (request.operationId === 'storageProviders.objects.content.update') {
    return {
      providerId: request.pathParams?.providerId ?? 'provider-s3',
      bucket: 'drive-prod',
      objectKind: 'object',
      objectKey: request.pathParams?.objectKey ?? 'docs/new.txt',
      contentLength: 5,
      contentType: (request.body as { contentType?: string } | undefined)?.contentType ?? null,
    };
  }

  if (request.operationId === 'storageProviders.objects.copy') {
    const body = request.body as { sourceObjectKey?: string; destinationObjectKey?: string } | undefined;
    return {
      providerId: request.pathParams?.providerId ?? 'provider-s3',
      bucket: 'drive-prod',
      objectKey: body?.destinationObjectKey ?? 'docs/copied.txt',
      changed: true,
    };
  }

  return {
    id: request.pathParams?.providerId ?? 'provider-s3',
    providerKind: 's3_compatible',
    name: 'Amazon S3',
    endpointUrl: 'https://s3.us-east-1.amazonaws.com',
    bucket: 'drive-prod',
    pathStyle: false,
    status: 'active',
    version: 1,
    credentialConfigured: true,
  };
}

function lastCall(calls: DriveAdminStorageSdkRequest[]): DriveAdminStorageSdkRequest {
  const call = calls.at(-1);
  if (!call) {
    throw new Error('Expected a Drive admin storage SDK call.');
  }
  return call;
}

describe('storage provider admin service', () => {
  it('lists storage providers through the Drive admin storage SDK', async () => {
    const { calls, service } = createFakeService();

    const providers = await service.listProviders({ status: 'active' });

    expect(providers).toHaveLength(1);
    expect(providers[0]).toMatchObject({
      id: 'provider-cos',
      providerKind: 'tencent_cos',
      displayName: 'Tencent COS',
      bucket: 'drive-prod',
      credentialConfigured: true,
    });
    expect(lastCall(calls)).toMatchObject({
      operationId: 'storageProviders.list',
      query: { status: 'active' },
    });
  });

  it('creates provider configuration with operator attribution and credential refs only', async () => {
    const { calls, service } = createFakeService();

    await service.createProvider({
      id: 'provider-s3',
      providerKind: 's3_compatible',
      name: 'Amazon S3',
      endpointUrl: 'https://s3.us-east-1.amazonaws.com',
      region: 'us-east-1',
      bucket: 'drive-prod',
      pathStyle: false,
      credentialRef: 'secret/aws-s3',
      status: 'active',
    });

    expect(lastCall(calls)).toMatchObject({
      operationId: 'storageProviders.create',
      body: {
        id: 'provider-s3',
        providerKind: 's3_compatible',
        name: 'Amazon S3',
        endpointUrl: 'https://s3.us-east-1.amazonaws.com',
        region: 'us-east-1',
        bucket: 'drive-prod',
        pathStyle: false,
        credentialRef: 'secret/aws-s3',
        status: 'active',
      },
    });
    expect(JSON.stringify(lastCall(calls).body)).not.toMatch(/secretAccessKey|accessKeySecret|privateKey/i);
  });

  it('sets and clears space type bindings through the admin storage SDK', async () => {
    const { calls, service } = createFakeService();

    await service.setSpaceTypeBinding({ spaceType: 'personal', providerId: 'provider-s3' });
    await service.deleteSpaceTypeBinding('personal');

    expect(calls.map((call) => call.operationId)).toEqual([
      'storageProviderBindings.default.update',
      'storageProviderBindings.default.delete',
    ]);
    expect(calls[0]).toMatchObject({
      body: {
        providerId: 'provider-s3',
        spaceType: 'personal',
      },
    });
    expect(calls[1]).toMatchObject({
      query: {
        spaceType: 'personal',
      },
    });
  });

  it('updates, activates, deactivates, tests, deletes, rotates credentials, and sets default bindings', async () => {
    const { calls, service } = createFakeService();

    await service.updateProvider('provider-s3', { name: 'AWS Primary' });
    await service.activateProvider('provider-s3');
    await service.deactivateProvider('provider-s3');
    const reachable = await service.testProvider('provider-s3');
    await service.rotateCredential('provider-s3', 'secret/aws-rotated');
    await service.setDefaultBinding({ providerId: 'provider-s3', spaceId: 'space-100' });
    const deleted = await service.deleteProvider('provider-s3');

    expect(reachable).toBe(true);
    expect(deleted).toBe(true);
    expect(calls.map((call) => call.operationId)).toEqual([
      'storageProviders.update',
      'storageProviders.activate',
      'storageProviders.deactivate',
      'storageProviders.test',
      'storageProviders.credentials.rotate',
      'storageProviderBindings.default.update',
      'storageProviders.delete',
    ]);
    expect(calls[0]).toMatchObject({
      pathParams: { providerId: 'provider-s3' },
      body: { name: 'AWS Primary' },
    });
    expect(calls[4]).toMatchObject({
      pathParams: { providerId: 'provider-s3' },
      body: { credentialRef: 'secret/aws-rotated' },
    });
    expect(calls[5]).toMatchObject({
      body: {
        providerId: 'provider-s3',
        spaceId: 'space-100',
      },
    });
    expect(calls[6]).toMatchObject({
      pathParams: { providerId: 'provider-s3' },
    });
  });

  it('maps provider object list fields from the OpenAPI contract', async () => {
    const { service } = createFakeService();

    const result = await service.listObjects('provider-s3', { prefix: 'docs/' });

    expect(result.items).toEqual([
      expect.objectContaining({
        key: 'docs/',
        sizeBytes: 0,
        isFolder: true,
      }),
      expect.objectContaining({
        key: 'docs/readme.txt',
        sizeBytes: 2048,
        contentType: 'text/plain',
        isFolder: false,
      }),
    ]);
  });

  it('requires tenant and operator context before mutating provider administration', async () => {
    const client = {
      metadata: {},
      operations: {},
      setTokenManager: () => undefined,
      request: async () => ({}),
    } as unknown as DriveAdminStorageSdkClient;
    const service: StorageProviderAdminService = createStorageProviderAdminService({
      adminStorageSdkClient: client,
      getSession: () => ({ context: { tenantId: 'tenant-100', userId: 'user-100' } }),
    });

    await expect(service.createProvider({
      id: 'provider-s3',
      providerKind: 's3_compatible',
      name: 'Amazon S3',
      endpointUrl: 'https://s3.us-east-1.amazonaws.com',
      bucket: 'drive-prod',
    })).rejects.toThrow('Drive admin session context is missing tenantId or operatorId.');
  });

  it('treats a missing default binding as an unconfigured empty state', async () => {
    const client = {
      metadata: {},
      operations: {},
      setTokenManager: () => undefined,
      request: async (request: DriveAdminStorageSdkRequest) => {
        throw new DriveAdminStorageSdkError({
          operationId: request.operationId,
          status: 404,
          detail: 'default storage provider binding not found',
        });
      },
    } as unknown as DriveAdminStorageSdkClient;
    const service = createStorageProviderAdminService({
      adminStorageSdkClient: client,
      getSession: () => ({
        context: {
          tenantId: 'tenant-100',
          userId: 'user-100',
        },
      }),
    });

    await expect(service.getDefaultBinding()).resolves.toBeUndefined();
  });


  it('lists provider kinds from the catalog operation', async () => {
    const { calls, service } = createFakeService();

    const kinds = await service.listKinds();

    expect(calls[0].operationId).toBe('storageProviderKinds.list');
    expect(kinds).toHaveLength(2);
    expect(kinds[0]).toMatchObject({
      providerKind: 'aliyun_oss',
      displayName: 'Alibaba Cloud OSS',
      enabled: true,
      sortOrder: 4,
      configCount: 2,
    });
    expect(kinds[1].enabled).toBe(false);
  });

  it('toggles a provider kind enabled state', async () => {
    const { calls, service } = createFakeService();

    const updated = await service.setKindEnabled('aliyun_oss', false);

    expect(calls[0].operationId).toBe('storageProviderKinds.update');
    expect(calls[0].pathParams).toEqual({ providerKind: 'aliyun_oss' });
    expect(calls[0].body).toEqual({ enabled: false });
    expect(updated.enabled).toBe(false);
  });

  it('initializes the provider kind catalog', async () => {
    const { calls, service } = createFakeService();

    const kinds = await service.initializeKinds();

    expect(calls[0].operationId).toBe('storageProviderKinds.initialize');
    expect(kinds).toHaveLength(2);
  });

  it('reads object content through the content retrieve operation', async () => {
    const { calls, service } = createFakeService();

    const content = await service.readObjectContent('provider-s3', 'docs/readme.txt');

    expect(calls[0].operationId).toBe('storageProviders.objects.content.retrieve');
    expect(calls[0].pathParams).toEqual({ providerId: 'provider-s3', objectKey: 'docs/readme.txt' });
    expect(content).toEqual({
      providerId: 'provider-s3',
      bucket: 'drive-prod',
      objectKey: 'docs/readme.txt',
      contentType: 'text/plain',
      sizeBytes: 5,
      encoding: 'base64',
      content: 'aGVsbG8=',
      checksumSha256: '2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824',
    });
  });

  it('writes object content through the content update operation', async () => {
    const { calls, service } = createFakeService();

    const written = await service.writeObjectContent('provider-s3', 'docs/new.txt', {
      content: 'aGVsbG8=',
      encoding: 'base64',
      contentType: 'text/plain',
    });

    expect(calls[0].operationId).toBe('storageProviders.objects.content.update');
    expect(calls[0].body).toEqual({
      content: 'aGVsbG8=',
      encoding: 'base64',
      contentType: 'text/plain',
    });
    expect(written.key).toBe('docs/new.txt');
    expect(written.isFolder).toBe(false);
  });

  it('copies and renames objects through copy and delete operations', async () => {
    const { calls, service } = createFakeService();

    const copied = await service.copyObject('provider-s3', {
      sourceObjectKey: 'docs/readme.txt',
      destinationObjectKey: 'docs/copied.txt',
    });
    expect(calls[0].operationId).toBe('storageProviders.objects.copy');
    expect(calls[0].body).toEqual({
      sourceObjectKey: 'docs/readme.txt',
      destinationObjectKey: 'docs/copied.txt',
    });
    expect(copied).toEqual({
      providerId: 'provider-s3',
      bucket: 'drive-prod',
      objectKey: 'docs/copied.txt',
      changed: true,
    });

    const renamed = await service.renameObject('provider-s3', 'docs/readme.txt', 'docs/renamed.txt');
    expect(calls[1].operationId).toBe('storageProviders.objects.copy');
    expect(calls[2].operationId).toBe('storageProviders.objects.delete');
    expect(renamed).toBe(true);
  });
});
