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

  if (request.operationId === 'storageProviderAccountDefaults.create') {
    return {
      items: [
        {
          // 无凭证的服务商：只铺配置，不铸造账号，也就没有占位密钥。
          providerKind: 'local_filesystem',
          providerId: 'builtin-storage-provider-local-filesystem',
          providerCreated: true,
        },
        {
          providerKind: 'aliyun_oss',
          providerId: 'builtin-storage-provider-aliyun-oss',
          providerCreated: true,
          vendorCode: 'aliyun',
          providerAccountId: 'iampacct-018f-seeded',
          accountCode: 'builtin-aliyun-storage',
          accountCreated: true,
          credentialSeeded: true,
          // 服务端在铸造占位账号时同样知道这个厂商怎么称呼两半密钥，随响应带回，
          // 这样控制台就不必再维护第二份会漂移的表。
          credentialFields: {
            accessKeyLabel: 'AccessKey ID',
            secretKeyLabel: 'AccessKey Secret',
            defaultEnvAccessKey: 'ALIBABA_CLOUD_ACCESS_KEY_ID',
            defaultEnvSecretKey: 'ALIBABA_CLOUD_ACCESS_KEY_SECRET',
            consoleUrl: 'https://ram.console.aliyun.com/manage/ak',
          },
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

/** A client that records every request and answers from a per-call builder. */
function recordingClient(
  calls: DriveAdminStorageSdkRequest[],
  responseForCall: () => unknown,
): DriveAdminStorageSdkClient {
  return {
    metadata: {},
    operations: {},
    setTokenManager: () => undefined,
    async request<T>(request: DriveAdminStorageSdkRequest): Promise<T> {
      calls.push(request);
      return responseForCall() as T;
    },
  } as unknown as DriveAdminStorageSdkClient;
}

function createServiceWithClient(client: DriveAdminStorageSdkClient): StorageProviderAdminService {
  return createStorageProviderAdminService({
    adminStorageSdkClient: client,
    getSession: () => ({
      context: { tenantId: 'tenant-100', userId: 'user-100', actorId: 'operator-100' },
    }),
  });
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

  it('projects the item envelope the transport delivers for single resources', async () => {
    // The SDKWork transport unwraps `data`, so a single resource arrives as
    // `{ item: {...} }`. Reading the fields off the outer object type-checks and
    // returns `undefined` for every one of them, which is how a bound tenant came
    // to read as unconfigured and every provider test as unreachable.
    const calls: DriveAdminStorageSdkRequest[] = [];
    const responses: Record<string, unknown> = {
      'storageProviderBindings.default.retrieve': {
        item: {
          id: 'default:tenant:tenant-100',
          tenantId: 'tenant-100',
          providerId: 'provider-cos',
          bindingScope: 'tenant',
          purpose: 'primary',
          lifecycleStatus: 'active',
          version: 3,
          storageRootPrefix: 'sdkwork-drive/v1/tenants/tenant-100',
          storageProvider: {
            id: 'provider-cos',
            providerKind: 'tencent_cos',
            name: 'Tencent COS',
            endpointUrl: 'https://cos.ap-shanghai.myqcloud.com',
            bucket: 'drive-prod',
            pathStyle: false,
            status: 'active',
            version: 2,
            credentialConfigured: true,
          },
        },
      },
      'storageProviders.test': { item: { providerId: 'provider-cos', reachable: true } },
      'storageProviders.bucket.retrieve': {
        item: { providerId: 'provider-cos', bucket: 'drive-prod', exists: true },
      },
      'storageProviders.bucket.update': {
        item: { providerId: 'provider-cos', bucket: 'drive-prod', changed: true },
      },
      'storageProviders.create': {
        item: {
          id: 'provider-new',
          providerKind: 'tencent_cos',
          name: 'New COS',
          endpointUrl: 'https://cos.ap-shanghai.myqcloud.com',
          bucket: 'drive-new',
          pathStyle: false,
          status: 'active',
          version: 1,
          credentialConfigured: true,
        },
      },
    };
    const service = createServiceWithClient(
      recordingClient(calls, () => responses[calls.at(-1)?.operationId ?? '']),
    );

    const binding = await service.getDefaultBinding();
    expect(binding).toMatchObject({
      id: 'default:tenant:tenant-100',
      providerId: 'provider-cos',
      bindingScope: 'tenant',
      storageRootPrefix: 'sdkwork-drive/v1/tenants/tenant-100',
    });
    expect(binding?.storageProvider).toMatchObject({
      id: 'provider-cos',
      providerKind: 'tencent_cos',
      bucket: 'drive-prod',
    });

    await expect(service.testProvider('provider-cos')).resolves.toBe(true);
    await expect(service.headBucket('provider-cos')).resolves.toMatchObject({ exists: true });
    await expect(service.initializeBucket('provider-cos')).resolves.toMatchObject({
      bucket: 'drive-prod',
      changed: true,
    });
    await expect(
      service.createProvider({
        id: 'provider-new',
        providerKind: 'tencent_cos',
        name: 'New COS',
        endpointUrl: 'https://cos.ap-shanghai.myqcloud.com',
        bucket: 'drive-new',
      }),
    ).resolves.toMatchObject({ id: 'provider-new', displayName: 'New COS' });
  });

  it('narrows the binding list to one resolution step', async () => {
    const { calls, service } = createFakeService();

    await service.listBindingsPage({ bindingScope: 'space_type', pageSize: 200 });

    // The console renders a section per step; the unfiltered list is one page of
    // every step, so the filter has to be a server query.
    expect(lastCall(calls)).toMatchObject({
      operationId: 'storageProviderBindings.list',
      query: { binding_scope: 'space_type', page_size: 200 },
    });
  });

  it('sends the provider-kind switch as a server-side list filter', async () => {
    const { calls, service } = createFakeService();

    await service.listProvidersPage({
      providerKind: 'tencent_cos',
      pageSize: 20,
      pageToken: 'cursor-2',
    });

    // The filter has to travel with the request: applied to a fetched page it
    // can only report the page, which is how "select Tencent COS" came back
    // empty while the row sat on page 2. The wire name is the canonical
    // lower_snake_case `provider_kind` (`API_SPEC.md` §13).
    expect(lastCall(calls)).toMatchObject({
      operationId: 'storageProviders.list',
      query: {
        provider_kind: 'tencent_cos',
        page_size: 20,
        cursor: 'cursor-2',
      },
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

  it('forwards the scope slice of the account list as query parameters', async () => {    const { calls, service } = createFakeService();

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

  it('sends the account page window and reads the server continuation back', async () => {
    const calls: DriveAdminStorageSdkRequest[] = [];
    const service = createServiceWithClient(
      recordingClient(calls, () => ({
        items: [
          {
            id: 'iampacct-018f-page-2',
            scopeType: 'platform',
            vendorCode: 'tencent',
            accountCode: 'tencent-main-hyeu7e61',
            displayName: 'Tencent storage',
            accountType: 'long_term_key',
            environment: 'production',
            capabilityCodes: ['object_storage'],
            status: 'active',
            credentialConfigured: true,
            credentialCount: 1,
            version: 1,
          },
        ],
        // The account list is paginated: a client that drops `pageInfo` cannot
        // tell this page from a complete set, which is how an account that
        // sorts past the window becomes unreachable.
        pageInfo: { mode: 'cursor', hasMore: true, nextCursor: 'b3BhcXVlLWN1cnNvcg' },
      })),
    );

    const page = await service.listProviderAccountsPage({
      status: 'active',
      scopeType: 'platform',
      vendorCode: 'tencent',
      pageSize: 20,
      pageToken: 'b3BhcXVlLWN1cnNvcg==',
    });

    expect(lastCall(calls)).toMatchObject({
      operationId: 'storageProviderAccounts.list',
      query: {
        status: 'active',
        scopeType: 'platform',
        vendorCode: 'tencent',
        page_size: 20,
        cursor: 'b3BhcXVlLWN1cnNvcg==',
      },
    });
    expect(page.items).toHaveLength(1);
    expect(page.items[0]).toMatchObject({ id: 'iampacct-018f-page-2', vendorCode: 'tencent' });
    expect(page.hasMore).toBe(true);
    expect(page.nextPageToken).toBe('b3BhcXVlLWN1cnNvcg');
  });

  it('reads the whole vendor bucket inventory instead of one window of it', async () => {
    // 厂商清单是全量的，接口却按 offset 分页：只读第一页时，第 21 个之后的桶在页面上
    // 直接消失，而页面把它们当"这个配置下的所有桶"来展示（数量徽标、搜索都在这个前提
    // 下才成立）。所以服务必须跟着 nextCursor 读完。
    const calls: DriveAdminStorageSdkRequest[] = [];
    const pages: unknown[] = [
      {
        items: [
          { bucket: 'archive-2024', configured: false, creationDateEpochMs: Date.UTC(2024, 0, 2) },
          { bucket: 'drive-prod', configured: true, creationDateEpochMs: Date.UTC(2023, 4, 1) },
        ],
        pageInfo: { mode: 'cursor', hasMore: true, nextCursor: 'b3BhcXVlLWJ1Y2tldC1jdXJzb3I' },
      },
      {
        items: [{ bucket: 'media-2025', configured: false }],
        pageInfo: { mode: 'cursor', hasMore: false },
      },
    ];
    let pageIndex = 0;
    const service = createServiceWithClient({
      metadata: {},
      operations: {},
      setTokenManager: () => undefined,
      async request<T>(request: DriveAdminStorageSdkRequest): Promise<T> {
        calls.push(request);
        const page = pages[Math.min(pageIndex, pages.length - 1)];
        pageIndex += 1;
        return page as T;
      },
    } as unknown as DriveAdminStorageSdkClient);

    const buckets = await service.listBuckets('provider-s3');

    expect(buckets.map((bucket) => bucket.bucket)).toEqual([
      'archive-2024',
      'drive-prod',
      'media-2025',
    ]);
    expect(buckets[1]).toMatchObject({ configured: true });
    /*
     * 时间以 ISO 交给显示层，服务层不做本地化：曾经这里是 `toLocaleDateString()`，
     * 读到的是浏览器区域而不是控制台语言，于是英文界面出现中文格式的日期，还被列表
     * 按语言二次格式化一遍。这条断言把"边界只搬 ISO"钉住。
     */
    expect(buckets[0].creationDateIso).toBe(new Date(Date.UTC(2024, 0, 2)).toISOString());
    expect(buckets[2].creationDateIso).toBeUndefined();
    // 按声明上限读（1..=200），续页带上服务端给的游标。
    expect(calls).toHaveLength(2);
    expect(calls[0]).toMatchObject({
      operationId: 'storageProviders.buckets.list',
      pathParams: { providerId: 'provider-s3' },
      query: { page_size: 200 },
    });
    expect(calls[0].query?.cursor).toBeUndefined();
    expect(calls[1]).toMatchObject({
      operationId: 'storageProviders.buckets.list',
      query: { page_size: 200, cursor: 'b3BhcXVlLWJ1Y2tldC1jdXJzb3I' },
    });
  });

  it('stops following a bucket cursor that does not advance the window', async () => {
    // 服务端若把同一个游标再回答一次，继续跟只会空转；同名桶也不该被记两遍
    // （offset 窗口在并发写入后可能重叠，重复行还会撞掉表格的 key）。
    const calls: DriveAdminStorageSdkRequest[] = [];
    const page = {
      items: [{ bucket: 'drive-prod', configured: true }],
      pageInfo: { mode: 'cursor', hasMore: true, nextCursor: 'c3RhbGxlZC1jdXJzb3I' },
    };
    const service = createServiceWithClient(
      recordingClient(calls, () => page),
    );

    const buckets = await service.listBuckets('provider-s3');

    expect(buckets.map((bucket) => bucket.bucket)).toEqual(['drive-prod']);
    expect(calls).toHaveLength(2);
  });

  it('reports a complete account page as complete instead of inferring it from a short page', async () => {
    const { calls, service } = createFakeService();

    const page = await service.listProviderAccountsPage({ scopeType: 'platform' });

    // The shared fake answers without `pageInfo` — the pre-fix server shape.
    // A missing continuation is the only honest reading of "no cursor".
    expect(page.hasMore).toBe(false);
    expect(page.nextPageToken).toBeUndefined();
    // The single-page wrapper still projects the items for callers that only
    // render one page (`listProviderAccounts`).
    expect(page.items[0]).toMatchObject({ id: 'iampacct-018f-list' });
    expect(calls.at(-1)?.query?.page_size).toBeUndefined();
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
        // 时间同样是 ISO：显示层按宿主语言格式化。
        lastModifiedIso: new Date(1700000000000).toISOString(),
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

  it('bootstraps built-in provider accounts and projects every per-row flag', async () => {
    const { calls, service } = createFakeService();

    const rows = await service.initializeProviderAccountDefaults();

    expect(calls[0].operationId).toBe('storageProviderAccountDefaults.create');
    // 纯副作用端点：请求体里没有身份，也没有任何凭证材料 —— 账号铸造与密钥密封全在
    // 服务端完成，客户端只提交"做这件事"这个意图。
    expect(calls[0].body).toBeUndefined();
    expect(calls[0].query).toBeUndefined();
    expect(calls[0].pathParams).toBeUndefined();

    expect(rows).toEqual([
      expect.objectContaining({
        providerKind: 'local_filesystem',
        providerId: 'builtin-storage-provider-local-filesystem',
        providerCreated: true,
        accountCreated: false,
        credentialSeeded: false,
      }),
      expect.objectContaining({
        providerKind: 'aliyun_oss',
        providerId: 'builtin-storage-provider-aliyun-oss',
        providerCreated: true,
        vendorCode: 'aliyun',
        providerAccountId: 'iampacct-018f-seeded',
        accountCode: 'builtin-aliyun-storage',
        accountCreated: true,
        credentialSeeded: true,
      }),
    ]);
    // 缺少账号字段的行不能被填成空串：`undefined` 表示"这个服务商没有账号"，
    // 空串会被凭证面板当成"有个 id 是空"的账号而拼出坏引用。
    expect(rows[0].providerAccountId).toBeUndefined();
    expect(rows[0].accountCode).toBeUndefined();
    expect(rows[0].vendorCode).toBeUndefined();
  });

  it('carries the server-side vendor credential vocabulary back to the console', async () => {
    const { service } = createFakeService();

    const rows = await service.initializeProviderAccountDefaults();

    // The server minted the placeholder key pair, so the labels it returns are
    // the authority; dropping them here is exactly the defect that let the
    // editor pre-fill an env name the bootstrapped account was not created
    // under.
    expect(rows[1].credentialFields).toEqual({
      accessKeyLabel: 'AccessKey ID',
      secretKeyLabel: 'AccessKey Secret',
      defaultEnvAccessKey: 'ALIBABA_CLOUD_ACCESS_KEY_ID',
      defaultEnvSecretKey: 'ALIBABA_CLOUD_ACCESS_KEY_SECRET',
      consoleUrl: 'https://ram.console.aliyun.com/manage/ak',
    });
    // A credential-free kind has no key pair, so it must not invent a
    // half-populated field object the editor could render.
    expect(rows[0].credentialFields).toBeUndefined();
  });

  it('degrades to no credential fields when the server omits a label', async () => {
    // Version skew: an older server returns the row without (or with a partial)
    // `credentialFields`. The console must fall back to its own static catalog
    // rather than hand the editor an object with empty strings.
    const client = {
      metadata: {},
      operations: {},
      setTokenManager: () => undefined,
      request: async () => ({
        items: [
          {
            providerKind: 'aliyun_oss',
            providerId: 'builtin-storage-provider-aliyun-oss',
            providerCreated: true,
            credentialFields: {
              accessKeyLabel: 'AccessKey ID',
              secretKeyLabel: '',
            },
          },
        ],
      }),
    } as unknown as DriveAdminStorageSdkClient;
    const service = createStorageProviderAdminService({
      adminStorageSdkClient: client,
      getSession: () => ({
        context: { tenantId: 'tenant-100', actorId: 'user-100', userId: 'user-100' },
      }),
    });

    const rows = await service.initializeProviderAccountDefaults();

    expect(rows[0].credentialFields).toBeUndefined();
  });

  it('requires an identified operator before bootstrapping provider accounts', async () => {
    const client = {
      metadata: {},
      operations: {},
      setTokenManager: () => undefined,
      request: async () => ({}),
    } as unknown as DriveAdminStorageSdkClient;
    const service = createStorageProviderAdminService({
      adminStorageSdkClient: client,
      getSession: () => ({ context: { tenantId: 'tenant-100', userId: 'user-100' } }),
    });

    await expect(service.initializeProviderAccountDefaults()).rejects.toThrow(
      'Drive admin session context is missing tenantId or operatorId.',
    );
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
