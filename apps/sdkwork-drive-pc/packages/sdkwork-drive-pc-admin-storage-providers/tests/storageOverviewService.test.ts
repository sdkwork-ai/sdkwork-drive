import { describe, expect, it } from 'vitest';
import type { DriveAdminStorageSdkRequest } from 'sdkwork-drive-pc-admin-core';
import { createStorageProviderAdminService } from '../src/services/storageProviderAdminService';

/**
 * 契约里 int64 一律是 string，所以这里故意用字符串喂一遍：任何一处忘了做
 * 字符串→数字兜底，断言就会拿到 NaN/0 而不是真实值。
 */
function overviewPayload(): Record<string, unknown> {
  return {
    generatedAt: '2026-09-24T12:00:00.000Z',
    scopeTenantId: 'tenant-001',
    capacity: {
      totalObjectCount: '120',
      activeObjectCount: '100',
      deletedObjectCount: '20',
      usedBytes: '1048576',
      averageObjectBytes: '10485',
      largestObjectBytes: '524288',
      bucketCount: '3',
      quotaBytes: '2097152',
      quotaConfigured: true,
      quotaUsageRatio: 0.5,
    },
    providers: {
      totalCount: '4',
      activeCount: '3',
      disabledCount: '1',
      deletedCount: '0',
      usage: [
        {
          providerId: 'provider-a',
          name: 'Provider A',
          providerKind: 'aliyun_oss',
          status: 'active',
          bucket: 'bucket-a',
          objectCount: '80',
          usedBytes: '838860',
          bindingCount: '2',
          isTenantDefault: true,
          capacityShare: 0.8,
        },
        {
          providerId: 'provider-b',
          name: 'Provider B',
          providerKind: 's3_compatible',
          status: 'disabled',
          bucket: 'bucket-b',
          objectCount: '20',
          usedBytes: '209716',
          bindingCount: '0',
          isTenantDefault: false,
          capacityShare: 0.2,
        },
      ],
    },
    bindings: {
      totalCount: '5',
      activeCount: '4',
      inactiveCount: '1',
      byScope: { tenantCount: '1', spaceCount: '3', spaceTypeCount: '1' },
      hasTenantDefault: true,
      tenantDefaultBindingId: 'default:tenant:tenant-001',
      tenantDefaultProviderId: 'provider-a',
    },
    catalog: { totalCount: '7', enabledCount: '6', disabledCount: '1' },
    trend: [
      { periodLabel: '2026-08', objectCount: '40', bytes: '409600' },
      { periodLabel: '2026-09', objectCount: '60', bytes: '638976' },
    ],
  };
}

function createHarness(payload: unknown) {
  const calls: DriveAdminStorageSdkRequest[] = [];
  const service = createStorageProviderAdminService({
    adminStorageSdkClient: {
      metadata: {},
      operations: {},
      setTokenManager: () => undefined,
      async request<T>(request: DriveAdminStorageSdkRequest): Promise<T> {
        calls.push(request);
        return payload as T;
      },
    } as never,
    getSession: () => ({ context: { tenantId: 'tenant-001', userId: 'u', actorId: 'o' } }) as never,
  });
  return { calls, service };
}

describe('storage overview service', () => {
  it('reads the aggregate through the overview operation and forwards the trend window', async () => {
    const { calls, service } = createHarness(overviewPayload());

    await service.getStorageOverview({ trendMonths: 24 });

    expect(calls).toHaveLength(1);
    expect(calls[0].operationId).toBe('storageOverview.retrieve');
    expect(calls[0].query).toEqual({ trendMonths: 24 });
  });

  it('omits the trend window when the caller leaves it unset', async () => {
    const { calls, service } = createHarness(overviewPayload());

    await service.getStorageOverview();

    // compactQuery() 会丢掉 undefined，所以服务端走自己的默认值（12 个月）。
    expect(calls[0].query).toEqual({});
  });

  it('coerces int64 string counters to numbers', async () => {
    const { service } = createHarness(overviewPayload());

    const overview = await service.getStorageOverview();

    expect(overview.capacity.usedBytes).toBe(1048576);
    expect(overview.capacity.activeObjectCount).toBe(100);
    expect(overview.capacity.quotaUsageRatio).toBe(0.5);
    expect(overview.providers.totalCount).toBe(4);
    expect(overview.providers.usage[0].objectCount).toBe(80);
    expect(overview.providers.usage[0].capacityShare).toBe(0.8);
    expect(overview.bindings.byScope.spaceCount).toBe(3);
    expect(overview.trend[1].bytes).toBe(638976);
    expect(overview.catalog.enabledCount).toBe(6);
  });

  it('exposes an optional quota as undefined and keeps the configured flag honest', async () => {
    const payload = overviewPayload();
    const capacity = payload.capacity as Record<string, unknown>;
    capacity.quotaBytes = null;
    capacity.quotaUsageRatio = null;
    capacity.quotaConfigured = false;
    capacity.largestObjectBytes = null;
    const { service } = createHarness(payload);

    const overview = await service.getStorageOverview();

    expect(overview.capacity.quotaBytes).toBeUndefined();
    expect(overview.capacity.quotaUsageRatio).toBeUndefined();
    expect(overview.capacity.quotaConfigured).toBe(false);
    expect(overview.capacity.largestObjectBytes).toBeUndefined();
  });

  it('tolerates a response that still carries the envelope item wrapper', async () => {
    const { service } = createHarness({ item: overviewPayload() });

    const overview = await service.getStorageOverview();

    expect(overview.scopeTenantId).toBe('tenant-001');
  });

  it('falls back to zeroed blocks instead of throwing on a malformed payload', async () => {
    const { service } = createHarness({});

    const overview = await service.getStorageOverview();

    expect(overview.capacity.usedBytes).toBe(0);
    expect(overview.providers.usage).toEqual([]);
    expect(overview.trend).toEqual([]);
    expect(overview.bindings.byScope.tenantCount).toBe(0);
  });
});
