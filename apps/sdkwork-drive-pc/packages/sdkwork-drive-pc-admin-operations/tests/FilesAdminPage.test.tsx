/* @vitest-environment jsdom */

import React from 'react';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { DriveBackendSdkClient } from 'sdkwork-drive-pc-admin-core';
import { LanguageProvider } from 'sdkwork-drive-pc-commons';
import { FilesAdminPage } from '../src/pages/FilesAdminPage';

const FIRST_FILE = {
  id: 'file-001',
  tenantId: 'tenant-001',
  spaceId: 'space-001',
  spaceName: 'Main',
  spaceType: 'personal',
  ownerSubjectType: 'user',
  ownerSubjectId: 'user-001',
  parentNodeId: null,
  nodeType: 'file',
  nodeName: 'report.pdf',
  filePath: 'Docs/report.pdf',
  fileExtension: 'pdf',
  contentType: 'application/pdf',
  contentTypeGroup: 'document',
  contentLength: '2048',
  lifecycleStatus: 'active',
  version: '3',
  createdBy: 'user-001',
  updatedBy: 'user-001',
  createdAt: '2026-11-14T00:00:00Z',
  updatedAt: '2026-11-14T01:00:00Z',
};

describe('FilesAdminPage', () => {
  it('renders the metadata projection and follows the server continuation cursor', async () => {
    const request = vi.fn(async ({ operationId }: { operationId: string }) => {
      if (operationId !== 'files.admin.list') {
        throw new Error(`Unexpected operation ${operationId}`);
      }

      if (request.mock.calls.length === 1) {
        return {
          items: [FIRST_FILE],
          pageInfo: {
            mode: 'cursor',
            pageSize: 20,
            totalItems: '2',
            hasMore: true,
            nextCursor: 'opaque-file-next',
          },
        };
      }

      return {
        items: [
          {
            ...FIRST_FILE,
            id: 'file-002',
            nodeName: 'archive.zip',
            filePath: 'Archive/archive.zip',
            contentTypeGroup: 'archive',
            contentLength: '0',
          },
        ],
        pageInfo: {
          mode: 'cursor',
          pageSize: 20,
          totalItems: '2',
          hasMore: false,
          nextCursor: null,
        },
      };
    });
    const backendSdkClient = { request } as unknown as DriveBackendSdkClient;

    render(
      <LanguageProvider defaultLanguage="en-US" resolveHostLanguage={() => 'en-US'}>
        <FilesAdminPage backendSdkClient={backendSdkClient} getSession={() => ({})} />
      </LanguageProvider>,
    );

    await screen.findByText('report.pdf');
    expect(screen.getByText('Docs/report.pdf')).toBeTruthy();
    expect(screen.getByText('Main')).toBeTruthy();
    expect(screen.getByText('space-001')).toBeTruthy();
    expect(screen.getByText('2.00 KB')).toBeTruthy();
    expect(screen.getByText('user-001')).toBeTruthy();
    expect(screen.getAllByText('Active').length).toBeGreaterThan(0);
    // The row renders a machine-readable timestamp; the human formatting is
    // locale/timezone dependent and is not what this test guards.
    expect(document.querySelector('time[datetime="2026-11-14T01:00:00Z"]')).toBeTruthy();

    fireEvent.click(screen.getByRole('button', { name: 'Next' }));

    await waitFor(() => expect(request).toHaveBeenCalledTimes(2));
    expect(request.mock.calls[1]?.[0]).toMatchObject({
      operationId: 'files.admin.list',
      query: {
        page_size: 20,
        cursor: 'opaque-file-next',
        lifecycle_status: 'active',
      },
    });
    await screen.findByText('archive.zip');
  });

  it('shows the int64 version and content metadata in the detail dialog without numeric coercion', async () => {
    const request = vi.fn(async () => ({
      items: [FIRST_FILE],
      pageInfo: { mode: 'cursor', pageSize: 20, totalItems: '1', hasMore: false },
    }));
    const backendSdkClient = { request } as unknown as DriveBackendSdkClient;

    render(
      <LanguageProvider defaultLanguage="en-US" resolveHostLanguage={() => 'en-US'}>
        <FilesAdminPage backendSdkClient={backendSdkClient} getSession={() => ({})} />
      </LanguageProvider>,
    );

    fireEvent.click(await screen.findByRole('button', { name: 'report.pdf' }));
    expect(screen.getByText('File details')).toBeTruthy();
    expect(screen.getByText('file-001')).toBeTruthy();
    expect(screen.getByText('3')).toBeTruthy();
    expect(screen.getByText('application/pdf')).toBeTruthy();

    fireEvent.keyDown(document, { key: 'Escape' });
    await waitFor(() => expect(screen.queryByText('File details')).toBeNull());
  });
});
