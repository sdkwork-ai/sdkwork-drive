/* @vitest-environment jsdom */

import React from 'react';
import { cleanup, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import type { DriveAdminStorageSdkClient } from 'sdkwork-drive-pc-admin-core';
import { StorageOverviewAdminPage } from '../src/pages/StorageOverviewAdminPage';

afterEach(() => cleanup());

const SESSION = () => ({
  context: { tenantId: 'tenant-001', userId: 'operator-001', actorId: 'operator-001' },
});

function fakeClient(request: (input: never) => Promise<unknown>) {
  return {
    metadata: {},
    operations: {},
    setTokenManager: () => undefined,
    request,
  } as unknown as DriveAdminStorageSdkClient;
}

const OVERVIEW = {
  generatedAt: '2026-09-24T12:00:00.000Z',
  scopeTenantId: 'tenant-001',
  capacity: {
    totalObjectCount: 120,
    activeObjectCount: 100,
    deletedObjectCount: 20,
    usedBytes: 1048576,
    averageObjectBytes: 10485,
    largestObjectBytes: 524288,
    bucketCount: 3,
    quotaBytes: 2097152,
    quotaConfigured: true,
    quotaUsageRatio: 0.5,
  },
  providers: {
    totalCount: 2,
    activeCount: 1,
    disabledCount: 1,
    deletedCount: 0,
    usage: [
      {
        providerId: 'provider-a',
        name: 'Provider A',
        providerKind: 'aliyun_oss',
        status: 'active',
        bucket: 'bucket-a',
        objectCount: 80,
        usedBytes: 838860,
        bindingCount: 2,
        isTenantDefault: true,
        capacityShare: 0.8,
      },
      {
        providerId: 'provider-b',
        name: 'Provider B',
        providerKind: 's3_compatible',
        status: 'disabled',
        bucket: 'bucket-b',
        objectCount: 20,
        usedBytes: 209716,
        bindingCount: 0,
        isTenantDefault: false,
        capacityShare: 0.2,
      },
    ],
  },
  bindings: {
    totalCount: 5,
    activeCount: 4,
    inactiveCount: 1,
    byScope: { tenantCount: 1, spaceCount: 3, spaceTypeCount: 1 },
    hasTenantDefault: true,
    tenantDefaultBindingId: 'default:tenant:tenant-001',
    tenantDefaultProviderId: 'provider-a',
  },
  catalog: { totalCount: 7, enabledCount: 6, disabledCount: 1 },
  trend: [
    { periodLabel: '2026-08', objectCount: 40, bytes: 409600 },
    { periodLabel: '2026-09', objectCount: 60, bytes: 638976 },
  ],
};

// 宿主没挂 LanguageProvider 时 `t()` 回退到原始 key，所以断言用的是 key 而不是翻译后的文案。
function renderPage(payload: unknown | Error) {
  render(
    <StorageOverviewAdminPage
      adminStorageSdkClient={fakeClient(async () => {
        if (payload instanceof Error) {
          throw payload;
        }
        return payload;
      })}
      getSession={SESSION as never}
    />,
  );
}

describe('StorageOverviewAdminPage', () => {
  it('renders the headline figures and the per-provider usage rows', async () => {
    renderPage(OVERVIEW);

    // 已用容量同时出现在 KPI 卡与配额条里，所以按“至少一处”断言。
    await waitFor(() => expect(screen.getAllByText('1.0 MB').length).toBeGreaterThan(0));
    // 活跃对象 / 待回收 / 在用配置 / 活跃绑定 / 已启用服务商 的 KPI 值
    for (const value of ['100', '20', '2', '4', '6']) {
      expect(screen.getAllByText(value).length).toBeGreaterThan(0);
    }
    // 用量明细行
    for (const value of ['Provider A', 'Provider B', 'bucket-a', 'provider-a']) {
      expect(screen.getAllByText(value).length).toBeGreaterThan(0);
    }
    // 默认绑定标记 + 停用标记
    expect(screen.getByText('overviewDefaultBadge')).toBeTruthy();
    // 停用标记同时出现在 provider 行徽标与绑定卡统计行。
    expect(screen.getAllByText('overviewStatusDisabled').length).toBeGreaterThan(0);
  });

  it('keeps the whole frame mounted when the tenant has stored nothing', async () => {
    const empty = {
      ...OVERVIEW,
      capacity: { ...OVERVIEW.capacity, usedBytes: 0, totalObjectCount: 0, activeObjectCount: 0, deletedObjectCount: 0, bucketCount: 0, largestObjectBytes: undefined, quotaUsageRatio: 0 },
      providers: { ...OVERVIEW.providers, totalCount: 0, activeCount: 0, disabledCount: 0, usage: [] },
      bindings: { ...OVERVIEW.bindings, totalCount: 0, activeCount: 0, inactiveCount: 0, hasTenantDefault: false, tenantDefaultBindingId: undefined, tenantDefaultProviderId: undefined },
      catalog: { totalCount: 7, enabledCount: 6, disabledCount: 1 },
      trend: OVERVIEW.trend.map((point) => ({ ...point, objectCount: 0, bytes: 0 })),
    };
    renderPage(empty);

    await waitFor(() => expect(screen.getByText('overviewEmptyTitle')).toBeTruthy());
    // 空态不是空白页：标题、说明、以及各区块框都还在
    expect(screen.getByText('overviewEmptyDesc')).toBeTruthy();
    expect(screen.getByText('overviewTrendEmpty')).toBeTruthy();
    expect(screen.getByText('overviewCapacityTitle')).toBeTruthy();
    expect(screen.getByText('overviewBindingHealthTitle')).toBeTruthy();
    // 缺默认绑定是运维要立刻处理的事，空态下依然要提示
    expect(screen.getByText('overviewDefaultBindingMissing')).toBeTruthy();
  });

  it('shows a retry affordance when the aggregate cannot be loaded', async () => {
    renderPage(new Error('boom'));

    await waitFor(() => expect(screen.getByText('overviewNoticeLoadFailed')).toBeTruthy());
    expect(screen.getByText('overviewRetry')).toBeTruthy();
  });
});
