/* @vitest-environment jsdom */

import React, { type ReactElement } from 'react';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { DriveBackendSdkClient } from 'sdkwork-drive-pc-admin-core';
import { LanguageProvider } from 'sdkwork-drive-pc-commons';
import { DownloadPackagesAdminPage } from '../src/pages/DownloadPackagesAdminPage';
import { LabelsAdminPage } from '../src/pages/LabelsAdminPage';
import { MaintenanceAdminPage } from '../src/pages/MaintenanceAdminPage';
import { QuotaAdminPage } from '../src/pages/QuotaAdminPage';

const session = () => ({
  context: {
    actorId: 'operator-001',
    tenantId: 'tenant-001',
    userId: 'operator-001',
  },
});

function renderChinese(element: ReactElement) {
  return render(
    <LanguageProvider defaultLanguage="zh-CN" resolveHostLanguage={() => 'zh-CN'}>
      {element}
    </LanguageProvider>,
  );
}

function offsetPage(items: unknown[]) {
  return {
    items,
    pageInfo: {
      mode: 'offset',
      page: 1,
      pageSize: 20,
      totalItems: String(items.length),
      totalPages: 1,
      hasMore: false,
    },
  };
}

afterEach(() => cleanup());

describe('Drive admin critical interactions', () => {
  it('requires confirmation before a live maintenance sweep', async () => {
    const request = vi.fn(async ({ operationId }: { operationId: string }) => {
      if (operationId === 'maintenance.jobs.list') return offsetPage([]);
      if (operationId === 'maintenance.objectSweep') {
        return { scannedCount: 20, affectedCount: 2, dryRun: false };
      }
      throw new Error(`Unexpected operation ${operationId}`);
    });
    const backendSdkClient = { request } as unknown as DriveBackendSdkClient;

    renderChinese(
      <MaintenanceAdminPage backendSdkClient={backendSdkClient} getSession={session} />,
    );

    await screen.findByText('暂无维护任务记录。');
    fireEvent.click(screen.getByRole('switch'));
    fireEvent.click(screen.getByRole('button', { name: '对象扫描' }));

    const dialog = screen.getByRole('dialog');
    expect(within(dialog).getByText('确认正式执行维护扫描？')).toBeTruthy();
    expect(request).not.toHaveBeenCalledWith(expect.objectContaining({
      operationId: 'maintenance.objectSweep',
    }));

    fireEvent.click(within(dialog).getByRole('button', { name: '正式执行' }));
    await waitFor(() => expect(request).toHaveBeenCalledWith(expect.objectContaining({
      operationId: 'maintenance.objectSweep',
      body: expect.objectContaining({ dryRun: false }),
    })));
  });

  it('requires confirmation before clearing the tenant quota policy', async () => {
    const request = vi.fn(async ({ operationId }: { operationId: string }) => {
      if (operationId === 'quotas.retrieve') {
        return { tenantId: 'tenant-001', totalBytes: 1024, objectCount: 3, quotaBytes: 4096 };
      }
      if (operationId === 'quotas.update') {
        return { tenantId: 'tenant-001', totalBytes: 1024, objectCount: 3, quotaBytes: null };
      }
      throw new Error(`Unexpected operation ${operationId}`);
    });
    const backendSdkClient = { request } as unknown as DriveBackendSdkClient;

    renderChinese(<QuotaAdminPage backendSdkClient={backendSdkClient} getSession={session} />);

    await screen.findByText('租户配额策略');
    fireEvent.click(screen.getByRole('button', { name: '清除策略' }));
    const dialog = screen.getByRole('dialog');
    expect(within(dialog).getByText('确认清除租户配额策略？')).toBeTruthy();

    fireEvent.click(within(dialog).getByRole('button', { name: '清除策略' }));
    await waitFor(() => expect(request).toHaveBeenCalledWith(expect.objectContaining({
      operationId: 'quotas.update',
      body: expect.objectContaining({ clearTenantPolicy: true }),
    })));
  });

  it('requires confirmation before deleting a label', async () => {
    const request = vi.fn(async ({ operationId }: { operationId: string }) => {
      if (operationId === 'labels.list') {
        return {
          items: [{
            id: 'label-confidential',
            tenantId: 'tenant-001',
            labelKey: 'confidential',
            displayName: '机密资料',
            color: '#DC2626',
            description: '仅授权成员可访问',
            lifecycleStatus: 'active',
            version: 1,
          }],
          pageInfo: { mode: 'cursor', pageSize: 20, hasMore: false },
        };
      }
      if (operationId === 'labels.delete') return undefined;
      throw new Error(`Unexpected operation ${operationId}`);
    });
    const backendSdkClient = { request } as unknown as DriveBackendSdkClient;

    renderChinese(<LabelsAdminPage backendSdkClient={backendSdkClient} getSession={session} />);

    await screen.findByText('机密资料');
    fireEvent.click(screen.getByRole('button', { name: '删除' }));
    const dialog = screen.getByRole('dialog');
    expect(within(dialog).getByText('确认删除标签？')).toBeTruthy();
    expect(within(dialog).getByText(/机密资料/)).toBeTruthy();

    fireEvent.click(within(dialog).getByRole('button', { name: '删除标签' }));
    await waitFor(() => expect(request).toHaveBeenCalledWith(expect.objectContaining({
      operationId: 'labels.delete',
      pathParams: { labelId: 'label-confidential' },
    })));
  });

  it('renders download package states from the Chinese catalog', async () => {
    const states = ['creating', 'ready', 'failed', 'expired'] as const;
    const request = vi.fn(async ({ operationId }: { operationId: string }) => {
      if (operationId !== 'downloadPackages.list') {
        throw new Error(`Unexpected operation ${operationId}`);
      }
      return offsetPage(states.map((state, index) => ({
        id: `package-${state}`,
        tenantId: 'tenant-001',
        packageName: `package-${index}.zip`,
        state,
        storageProviderId: 'provider-primary',
        bucket: 'drive-primary',
        archiveObjectKey: `exports/package-${index}.zip`,
        contentType: 'application/zip',
        fileCount: index + 1,
        totalBytes: 1024 * (index + 1),
        archiveSizeBytes: 1024 * (index + 1),
        expiresAtEpochMs: Date.now() + 60_000,
        createdBy: 'operator-001',
        updatedBy: 'operator-001',
        createdAt: '2026-07-20T10:00:00.000Z',
        updatedAt: '2026-07-20T10:00:00.000Z',
      })));
    });
    const backendSdkClient = { request } as unknown as DriveBackendSdkClient;

    renderChinese(
      <DownloadPackagesAdminPage backendSdkClient={backendSdkClient} getSession={session} />,
    );

    for (const label of ['创建中', '就绪', '失败', '已过期']) {
      expect((await screen.findAllByText(label)).length).toBeGreaterThan(0);
    }
    for (const state of states) {
      expect(screen.queryByText(state)).toBeNull();
    }
  });
});
