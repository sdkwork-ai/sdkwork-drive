/* @vitest-environment jsdom */

import React from 'react';
import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { DriveBackendSdkClient } from 'sdkwork-drive-pc-admin-core';
import { LanguageProvider, type Language } from 'sdkwork-drive-pc-commons';
import { MaintenanceAdminPage } from '../src/pages/MaintenanceAdminPage';

const maintenanceJob = {
  id: 1,
  jobType: 'object_sweep',
  status: 'completed',
  dryRun: true,
  scannedCount: 12,
  affectedCount: 3,
  operatorId: 'operator-001',
  startedAt: '2026-07-20T10:00:00.000Z',
  finishedAt: '2026-07-20T10:00:01.000Z',
  createdAt: '2026-07-20T10:00:00.000Z',
} as const;

function renderMaintenancePage(language: Language) {
  const request = vi.fn(async ({ operationId }: { operationId: string }) => {
    if (operationId !== 'maintenance.jobs.list') {
      throw new Error(`Unexpected operation ${operationId}`);
    }

    return {
      items: [maintenanceJob],
      pageInfo: {
        mode: 'offset',
        page: 1,
        pageSize: 20,
        totalItems: '1',
        totalPages: 1,
        hasMore: false,
      },
    };
  });
  const backendSdkClient = { request } as unknown as DriveBackendSdkClient;

  render(
    <LanguageProvider
      defaultLanguage={language}
      resolveHostLanguage={() => language}
    >
      <MaintenanceAdminPage
        backendSdkClient={backendSdkClient}
        getSession={() => ({})}
      />
    </LanguageProvider>,
  );

  return request;
}

describe('MaintenanceAdminPage', () => {
  it.each([
    {
      language: 'zh-CN' as const,
      buttons: ['对象扫描', '上传会话扫描', '过期上传内容扫描', '废弃上传任务扫描'],
      status: '已完成',
    },
    {
      language: 'en-US' as const,
      buttons: [
        'Object sweep',
        'Upload session sweep',
        'Expired upload content sweep',
        'Abandoned upload task sweep',
      ],
      status: 'Completed',
    },
  ])('localizes maintenance action buttons in $language', async ({ language, buttons, status }) => {
    const request = renderMaintenancePage(language);

    for (const button of buttons) {
      expect(screen.getByRole('button', { name: button })).toBeTruthy();
    }
    expect(await screen.findByText(status, { exact: false })).toBeTruthy();
    expect(request).toHaveBeenCalledWith(expect.objectContaining({
      operationId: 'maintenance.jobs.list',
    }));
    expect(screen.queryByText(/adminOperations\.jobType\./)).toBeNull();
  });
});
