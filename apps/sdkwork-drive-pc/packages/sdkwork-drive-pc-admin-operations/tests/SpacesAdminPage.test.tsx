/* @vitest-environment jsdom */

import React from 'react';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { DriveBackendSdkClient } from 'sdkwork-drive-pc-admin-core';
import { LanguageProvider } from 'sdkwork-drive-pc-commons';
import { SpacesAdminPage } from '../src/pages/SpacesAdminPage';

describe('SpacesAdminPage', () => {
  it('uses the server returned opaque cursor when loading the next spaces page', async () => {
    const request = vi.fn(async ({ operationId }: { operationId: string }) => {
      if (operationId !== 'spaces.admin.list') {
        throw new Error(`Unexpected operation ${operationId}`);
      }

      if (request.mock.calls.length === 1) {
        return {
          items: [{
            id: 'space-001',
            tenantId: 'tenant-001',
            ownerSubjectType: 'user',
            ownerSubjectId: 'user-001',
            displayName: 'Primary',
            spaceType: 'personal',
            lifecycleStatus: 'active',
            version: 1,
          }],
          pageInfo: {
            mode: 'cursor',
            pageSize: 20,
            totalItems: '40',
            hasMore: true,
            nextCursor: 'opaque-next',
          },
        };
      }

      return {
        items: [{
          id: 'space-002',
          tenantId: 'tenant-001',
          ownerSubjectType: 'user',
          ownerSubjectId: 'user-002',
          displayName: 'Archive',
          spaceType: 'personal',
          lifecycleStatus: 'active',
          version: 1,
        }],
        pageInfo: {
          mode: 'cursor',
          pageSize: 20,
          totalItems: '40',
          hasMore: false,
        },
      };
    });
    const backendSdkClient = { request } as unknown as DriveBackendSdkClient;

    render(
      <LanguageProvider defaultLanguage="en-US" resolveHostLanguage={() => 'en-US'}>
        <SpacesAdminPage
          backendSdkClient={backendSdkClient}
          getSession={() => ({})}
        />
      </LanguageProvider>,
    );

    await screen.findByText('Primary');
    expect(screen.getByText('space-001')).toBeTruthy();
    expect(screen.getByText('Personal')).toBeTruthy();
    expect(screen.getByText('User')).toBeTruthy();
    expect(screen.getByText('user-001')).toBeTruthy();
    expect(screen.getByText('Active')).toBeTruthy();

    fireEvent.click(screen.getByRole('button', { name: 'Next' }));

    await waitFor(() => expect(request).toHaveBeenCalledTimes(2));
    expect(request.mock.calls[1]?.[0]).toMatchObject({
      operationId: 'spaces.admin.list',
      query: {
        page_size: 20,
        cursor: 'opaque-next',
      },
    });
  });
});
