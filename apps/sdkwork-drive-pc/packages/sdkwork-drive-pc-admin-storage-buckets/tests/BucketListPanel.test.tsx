/* @vitest-environment jsdom */

import React from 'react';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { DriveAdminStorageSdkClient } from 'sdkwork-drive-pc-admin-core';
import { createStorageProviderAdminService } from 'sdkwork-drive-pc-admin-storage-providers';
import { LanguageProvider } from 'sdkwork-drive-pc-commons';
import { BucketListPanel } from '../src/components/BucketListPanel';

afterEach(() => cleanup());

function createFakeClient(request: unknown) {
  return {
    metadata: {},
    operations: {},
    setTokenManager: () => undefined,
    request,
  } as unknown as DriveAdminStorageSdkClient;
}

const getSession = () => ({
  context: { tenantId: 'tenant-100', userId: 'user-100', actorId: 'operator-100' },
});

const PROVIDER = {
  id: 'provider-cos-b',
  providerKind: 'tencent_cos',
  displayName: 'COS Guangzhou',
  endpointUrl: 'https://cos.ap-guangzhou.myqcloud.com',
  region: 'ap-guangzhou',
  bucket: 'drive-guangzhou',
  pathStyle: false,
  credentialConfigured: true,
  status: 'active',
  version: 1,
  strictTls: true,
};

const CREATION_EPOCH_MS = Date.UTC(2023, 4, 1);

/**
 * 账号级清单的两种厂商答案都要有：AWS 这种报了地域的，和 COS 这种只字未提的
 * （历史上正是这种让"所属地域"整列都是「—」）。
 */
function createRequest(items: unknown[]) {
  return vi.fn(async ({ operationId }: { operationId?: string }) => {
    if (operationId === 'storageProviders.buckets.list') {
      return { items, pageInfo: { mode: 'cursor', hasMore: false } };
    }
    throw new Error(`Unexpected operation ${operationId}`);
  });
}

function renderPanel(
  items: unknown[],
  locale: 'zh-CN' | 'en-US' = 'zh-CN',
) {
  const onBrowse = vi.fn();
  const request = createRequest(items);
  const service = createStorageProviderAdminService({
    adminStorageSdkClient: createFakeClient(request),
    getSession,
  });
  render(
    <LanguageProvider defaultLanguage={locale} resolveHostLanguage={() => locale}>
      <BucketListPanel onBrowse={onBrowse} provider={PROVIDER} service={service} />
    </LanguageProvider>,
  );
  return { onBrowse, request };
}

describe('BucketListPanel rows', () => {
  it('gives every row a role, a region answer and a localized date', async () => {
    renderPanel([
      {
        bucket: 'drive-guangzhou',
        configured: true,
        creationDateEpochMs: CREATION_EPOCH_MS,
        region: 'ap-guangzhou',
      },
      {
        bucket: 'archive-2024',
        configured: false,
        creationDateEpochMs: CREATION_EPOCH_MS,
        region: 'cn-beijing',
      },
      { bucket: 'media-2024', configured: false },
    ]);

    // 用途列：每行都有值（旧"状态"列只有配置桶那行有值，其余全是「—」）。
    expect(await screen.findByText('写入目标')).toBeTruthy();
    expect(screen.getAllByText('仅浏览')).toHaveLength(2);

    // 地域列只认接口返回的值：厂商报了的用厂商的，并给出本地化地域名（不只丢一个
    // 原始 code）；字典里没有该 code 的厂商区域就直接显示 code，不编造名字。
    expect(await screen.findByText('华南（广州） ap-guangzhou')).toBeTruthy();
    expect(await screen.findByText('cn-beijing')).toBeTruthy();
    // 厂商没报就留空——配置地域不是这个桶的地域，不能拿来顶替。
    const rows = screen.getAllByRole('row');
    const noRegionRow = rows.find((row) => row.textContent?.includes('media-2024')) as HTMLElement;
    expect(noRegionRow.textContent).toContain('—');
    expect(noRegionRow.textContent).not.toContain('ap-guangzhou');

    // 创建时间按宿主语言格式化，并且带机器可读的 <time dateTime>。
    const time = document.querySelector('time');
    expect(time?.getAttribute('dateTime')).toBe(new Date(CREATION_EPOCH_MS).toISOString());
    expect(time?.textContent).toBe(
      new Intl.DateTimeFormat('zh-CN', { year: 'numeric', month: 'short', day: 'numeric' })
        .format(new Date(CREATION_EPOCH_MS)),
    );
  });

  it('formats the same timestamp differently per console language', async () => {
    const items = [
      { bucket: 'archive-2024', configured: false, creationDateEpochMs: CREATION_EPOCH_MS },
    ];
    renderPanel(items, 'zh-CN');
    const chinese = (await screen.findByText(/2023/)).textContent;
    cleanup();

    renderPanel(items, 'en-US');
    const english = (await screen.findByText(/2023/)).textContent;

    // 这条断言就是"页面国际化"的回归守卫：以前服务层用 toLocaleDateString()，
    // 两种语言拿到的是同一个浏览器区域格式。
    expect(english).not.toBe(chinese);
    expect(english).toContain('May');
  });

  it('opens the bucket from the row, and sends only the vendor-reported region', async () => {
    const { onBrowse } = renderPanel([
      { bucket: 'media-2024', configured: false, region: 'cn-beijing' },
      { bucket: 'archive-2024', configured: false },
    ]);

    // 厂商报了地域：随行带走，读文件才会落到这个桶自己的端点。
    fireEvent.click(await screen.findByText('media-2024'));
    expect(onBrowse).toHaveBeenCalledWith('media-2024', 'cn-beijing');

    // 厂商没报：不带地域（服务端自会退回配置端点），绝不把配置地域冒充成桶的地域。
    fireEvent.click(screen.getByText('archive-2024'));
    expect(onBrowse).toHaveBeenLastCalledWith('archive-2024', undefined);
  });

  it('opens the focused row with the keyboard, like the file manager does', async () => {
    const { onBrowse } = renderPanel([
      { bucket: 'archive-2024', configured: false, region: 'cn-beijing' },
    ]);

    const row = (await screen.findByText('archive-2024')).closest('tr') as HTMLElement;
    expect(row.tabIndex).toBe(0);
    fireEvent.keyDown(row, { key: 'Enter' });
    await waitFor(() => {
      expect(onBrowse).toHaveBeenCalledWith('archive-2024', 'cn-beijing');
    });

    onBrowse.mockClear();
    fireEvent.keyDown(row, { key: ' ' });
    await waitFor(() => {
      expect(onBrowse).toHaveBeenCalledWith('archive-2024', 'cn-beijing');
    });
  });

  it('keeps the row button from firing the action twice', async () => {
    const { onBrowse } = renderPanel([
      { bucket: 'archive-2024', configured: false, region: 'cn-beijing' },
    ]);

    fireEvent.click(await screen.findByRole('button', { name: '管理文件' }));
    expect(onBrowse).toHaveBeenCalledTimes(1);
  });
});
