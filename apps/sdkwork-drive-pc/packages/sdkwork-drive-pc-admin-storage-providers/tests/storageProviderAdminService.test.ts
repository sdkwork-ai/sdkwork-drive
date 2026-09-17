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
  if (request.operationId === 'storageProviderKinds.list' || request.operationId === 'storageProviderKinds.create') {
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

  if (request.operationId === 'storageProviderAccounts.list') {
    return {
      items: [
        {
          id: 'iampacct-018f-list',
          vendorCode: 'aliyun',
          accountCode: 'aliyun-main-a1b2c3d4',
          displayName: 'Aliyun main account',
          accountType: 'standard',
          environment: 'production',
          capabilityCodes: ['object_storage'],
          status: 'active',
          credentialConfigured: true,
          credentialCount: 1,
          version: 1,
        },
        {
          // A platform-wide default: the row a tenant is meant to reuse without
          // having created it. It carries `tenant_id` = 100001 on the server, so
          // the client must surface the scope rather than infer it from the tenant.
          id: 'iampacct-018f-platform',
          scopeType: 'platform',
          isDefault: true,
          vendorCode: 'aliyun',
          accountCode: 'aliyun-platform-main',
          displayName: 'Aliyun platform account',
          accountType: 'standard',
          environment: 'production',
          capabilityCodes: ['object_storage'],
          status: 'active',
          credentialConfigured: true,
          credentialCount: 1,
          version: 3,
        },
        {
          // A personal account: listed so the operator can see it, but never
          // bindable to a tenant-level provider.
          id: 'iampacct-018f-mine',
          scopeType: 'user',
          ownerUserId: 'user-100',
          isDefault: false,
          vendorCode: 'aliyun',
          accountCode: 'aliyun-personal',
          displayName: 'My own Aliyun account',
          accountType: 'standard',
          environment: 'production',
          capabilityCodes: ['object_storage'],
          status: 'active',
          credentialConfigured: true,
          credentialCount: 1,
          version: 1,
        },
      ],
    };
  }

  if (request.operationId === 'storageProviderAccounts.create') {
    return {
      item: {
        id: 'iampacct-018f-created',
        vendorCode: (request.body as { vendorCode?: string } | undefined)?.vendorCode ?? 'aliyun',
        accountCode: 'aliyun-main-a1b2c3d4',
        displayName: 'Aliyun main account',
        accountType: 'standard',
        environment: 'production',
        capabilityCodes: ['object_storage'],
        status: 'active',
        credentialConfigured: true,
        credentialCount: 1,
        version: 1,
      },
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

  it('lists reusable provider accounts through the account center operation', async () => {
    const { calls, service } = createFakeService();

    const accounts = await service.listProviderAccounts({
      vendorCode: 'aliyun',
      status: 'active',
      capabilityCode: 'object_storage',
    });

    expect(accounts[0]).toMatchObject({
      id: 'iampacct-018f-list',
      vendorCode: 'aliyun',
      accountCode: 'aliyun-main-a1b2c3d4',
      credentialConfigured: true,
    });
    expect(lastCall(calls)).toMatchObject({
      operationId: 'storageProviderAccounts.list',
      query: {
        vendorCode: 'aliyun',
        status: 'active',
        capabilityCode: 'object_storage',
      },
    });
  });

  it('projects the account scope and the default flag onto every listed account', async () => {
    const { service } = createFakeService();

    const accounts = await service.listProviderAccounts();

    // A row that predates scopes (or a non-scope-aware server) must not surface
    // as `undefined`: the column default is `tenant` and the view type is a
    // closed union, so the projection has to converge on a member.
    expect(accounts[0]).toMatchObject({ id: 'iampacct-018f-list', scopeType: 'tenant', isDefault: false });
    expect(accounts[0].ownerUserId).toBeUndefined();
    expect(accounts[1]).toMatchObject({
      id: 'iampacct-018f-platform',
      scopeType: 'platform',
      isDefault: true,
    });
    expect(accounts[2]).toMatchObject({
      id: 'iampacct-018f-mine',
      scopeType: 'user',
      ownerUserId: 'user-100',
      isDefault: false,
    });
  });

  it('forwards the scope slice of the account list as query parameters', async () => {
    const { calls, service } = createFakeService();

    await service.listProviderAccounts({ mine: true });
    expect(lastCall(calls).query).toMatchObject({ mine: true });
    // `mine` and `scopeType` are mutually exclusive views: sending both would
    // make the server resolve an owner filter against a pinned scope. The key may
    // be present-but-undefined here — the SDK transport's `compactQuery()`
    // drops `undefined` values, so no empty parameter ever reaches the wire.
    expect(lastCall(calls).query?.scopeType).toBeUndefined();

    await service.listProviderAccounts({ scopeType: 'platform', includePlatform: true });
    expect(lastCall(calls).query).toMatchObject({ scopeType: 'platform', includePlatform: true });
    expect(lastCall(calls).query?.mine).toBeUndefined();

    // An unfiltered call must not pin any scope, so the server applies its own
    // defaults (every visible level, platform included).
    await service.listProviderAccounts();
    expect(lastCall(calls).query?.scopeType).toBeUndefined();
    expect(lastCall(calls).query?.mine).toBeUndefined();
    expect(lastCall(calls).query?.includePlatform).toBeUndefined();
  });

  it('registers a reusable account with its access key pair through the account center', async () => {    const { calls, service } = createFakeService();

    const account = await service.createProviderAccount({
      displayName: 'Aliyun main account',
      vendorCode: 'aliyun',
      accountCode: 'aliyun-main-a1b2c3d4',
      accessKeyId: 'AKID-example',
      secretAccessKey: 'secret-material',
    });

    expect(account).toMatchObject({
      id: 'iampacct-018f-created',
      vendorCode: 'aliyun',
      credentialConfigured: true,
    });
    expect(lastCall(calls)).toMatchObject({
      operationId: 'storageProviderAccounts.create',
      body: {
        displayName: 'Aliyun main account',
        vendorCode: 'aliyun',
        accountCode: 'aliyun-main-a1b2c3d4',
        accessKeyId: 'AKID-example',
        secretAccessKey: 'secret-material',
      },
    });
    // The sealed credential is write-only: the registration response never
    // echoes secret material back.
    expect(JSON.stringify(account)).not.toMatch(/secret-material/);
  });

  it('carries the requested account scope and default flag into the create body', async () => {
    const { calls, service } = createFakeService();

    await service.createProviderAccount({
      displayName: 'Aliyun platform account',
      vendorCode: 'aliyun',
      accountCode: 'aliyun-platform-main',
      scopeType: 'platform',
      isDefault: true,
      accessKeyId: 'AKID-example',
      secretAccessKey: 'secret-material',
    });

    expect(lastCall(calls)).toMatchObject({
      operationId: 'storageProviderAccounts.create',
      body: { scopeType: 'platform', isDefault: true },
    });

    // Omitting the scope must leave the field absent so the server applies its
    // own default (`tenant`) instead of receiving an explicit `undefined`.
    await service.createProviderAccount({
      displayName: 'Tenant account',
      vendorCode: 'aliyun',
      accountCode: 'aliyun-tenant-main',
      accessKeyId: 'AKID-example',
      secretAccessKey: 'secret-material',
    });
    expect(lastCall(calls).body).toBeDefined();
    expect((lastCall(calls).body as Record<string, unknown>).scopeType).toBeUndefined();
    expect((lastCall(calls).body as Record<string, unknown>).isDefault).toBeUndefined();
  });

  it('passes providerAccountId through provider create and update bodies', async () => {
    const { calls, service } = createFakeService();

    await service.createProvider({
      id: 'provider-s3',
      providerKind: 's3_compatible',
      name: 'Amazon S3',
      endpointUrl: 'https://s3.us-east-1.amazonaws.com',
      bucket: 'drive-prod',
      providerAccountId: 'iampacct-018f-created',
    });
    await service.updateProvider('provider-s3', {
      providerAccountId: 'iampacct-018f-created',
      credentialRef: '',
    });

    expect(calls[0]).toMatchObject({
      operationId: 'storageProviders.create',
      body: { providerAccountId: 'iampacct-018f-created' },
    });
    expect(calls[1]).toMatchObject({
      operationId: 'storageProviders.update',
      body: { providerAccountId: 'iampacct-018f-created', credentialRef: '' },
    });
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

  it('forwards every editable provider field on update, strict TLS included', async () => {
    // Regression: `updateProvider` built its body field-by-field and dropped
    // `strictTls`, so toggling "强制 TLS (仅 HTTPS)" in the edit form silently
    // reverted to the server-side default while every other field persisted.
    const { calls, service } = createFakeService();

    await service.updateProvider('provider-s3', {
      name: 'AWS Primary',
      endpointUrl: 'https://s3.us-east-1.amazonaws.com',
      region: 'us-east-1',
      bucket: 'drive-prod',
      pathStyle: true,
      strictTls: false,
      credentialRef: 'secret/aws-s3',
      serverSideEncryptionMode: 'AES256',
      defaultStorageClass: 'STANDARD',
      status: 'active',
    });

    expect(lastCall(calls)).toMatchObject({
      operationId: 'storageProviders.update',
      pathParams: { providerId: 'provider-s3' },
      body: {
        name: 'AWS Primary',
        endpointUrl: 'https://s3.us-east-1.amazonaws.com',
        region: 'us-east-1',
        bucket: 'drive-prod',
        pathStyle: true,
        strictTls: false,
        credentialRef: 'secret/aws-s3',
        serverSideEncryptionMode: 'AES256',
        defaultStorageClass: 'STANDARD',
        status: 'active',
      },
    });
  });

  it('omits untouched fields on update so the server keeps their current values', async () => {
    const { calls, service } = createFakeService();

    await service.updateProvider('provider-s3', { name: 'AWS Primary' });

    expect(lastCall(calls).body).toEqual({ name: 'AWS Primary' });
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

    expect(calls[0].operationId).toBe('storageProviderKinds.create');
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
