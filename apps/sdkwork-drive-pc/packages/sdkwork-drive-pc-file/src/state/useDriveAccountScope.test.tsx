/* @vitest-environment jsdom */

import React from 'react';
import { act, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  createSessionStore,
  type DriveFileService,
  type SessionStore,
} from 'sdkwork-drive-pc-core';

import {
  getDriveCutSelection,
  setDriveCutClipboardScope,
  setDriveCutSelection,
} from './driveCutClipboard';
import { useDriveAccountScope, type DriveAccountScope } from './useDriveAccountScope';

function createFileServiceStub(): { service: DriveFileService; syncAccountScope: ReturnType<typeof vi.fn> } {
  const syncAccountScope = vi.fn();
  const service = { syncAccountScope } as unknown as DriveFileService;
  return { service, syncAccountScope };
}

function createMemorySessionStore(): SessionStore {
  const items = new Map<string, string>();
  return createSessionStore({
    getItem: (key) => items.get(key) ?? null,
    setItem: (key, value) => {
      items.set(key, value);
    },
    removeItem: (key) => {
      items.delete(key);
    },
  });
}

function AccountScopeProbe({
  session,
  fileService,
  onState,
}: {
  session: SessionStore;
  fileService: DriveFileService;
  onState: (state: DriveAccountScope) => void;
}) {
  const state = useDriveAccountScope(session, fileService);
  onState(state);
  return <div data-testid="scope-key">{state.scopeKey}</div>;
}

function renderProbe(session: SessionStore, fileService: DriveFileService) {
  let latest: DriveAccountScope | undefined;
  const view = render(
    <AccountScopeProbe
      session={session}
      fileService={fileService}
      onState={(state) => {
        latest = state;
      }}
    />,
  );
  return {
    ...view,
    getState: (): DriveAccountScope => {
      if (!latest) {
        throw new Error('Account scope probe did not render.');
      }
      return latest;
    },
  };
}

const signedInSession = {
  authToken: 'auth-token',
  accessToken: 'access-token',
  context: {
    tenantId: 'tenant-001',
    userId: 'user-001',
  },
};

afterEach(() => {
  setDriveCutClipboardScope(undefined);
});

describe('useDriveAccountScope', () => {
  it('reports no owner and no persistence while signed out', () => {
    const session = createMemorySessionStore();
    const { service, syncAccountScope } = createFileServiceStub();
    const probe = renderProbe(session, service);

    expect(probe.getState()).toMatchObject({
      scope: undefined,
      scopeKey: '',
      signedIn: false,
      canPersist: false,
    });
    expect(screen.getByTestId('scope-key').textContent).toBe('');
    expect(syncAccountScope).toHaveBeenCalledWith('');
  });

  it('scopes persistence to the tenant and user of the signed-in account', () => {
    const session = createMemorySessionStore();
    const { service, syncAccountScope } = createFileServiceStub();
    session.setSession(signedInSession);

    const probe = renderProbe(session, service);

    expect(probe.getState()).toMatchObject({
      scope: { tenantId: 'tenant-001', userId: 'user-001' },
      scopeKey: 'tenant-001:user-001',
      signedIn: true,
      canPersist: true,
    });
    expect(syncAccountScope).toHaveBeenCalledWith('tenant-001:user-001');
  });

  it('re-invalidates the file service when another account signs in', () => {
    const session = createMemorySessionStore();
    const { service, syncAccountScope } = createFileServiceStub();
    session.setSession(signedInSession);
    const probe = renderProbe(session, service);

    act(() => {
      session.setSession({
        authToken: 'auth-token-b',
        accessToken: 'access-token-b',
        context: { tenantId: 'tenant-001', userId: 'user-002' },
      });
    });

    expect(probe.getState()).toMatchObject({
      scopeKey: 'tenant-001:user-002',
      canPersist: true,
    });
    expect(syncAccountScope).toHaveBeenLastCalledWith('tenant-001:user-002');
  });

  it('keeps account-scoped state in memory when the session has no tenant context', () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => undefined);
    const session = createMemorySessionStore();
    const { service } = createFileServiceStub();
    session.setSession({ authToken: 'auth-token', accessToken: 'access-token' });

    const probe = renderProbe(session, service);

    expect(probe.getState()).toMatchObject({
      scope: undefined,
      scopeKey: '',
      signedIn: true,
      canPersist: false,
    });
    expect(warn).toHaveBeenCalledTimes(1);
    expect(String(warn.mock.calls[0]?.[0])).toContain('tenant/user context');
    warn.mockRestore();
  });

  it('drops the previous account pending cut when the account changes', () => {
    setDriveCutClipboardScope('tenant-001:user-001');
    setDriveCutSelection({
      section: 'my-storage',
      parentId: null,
      files: [
        {
          id: 'node-1',
          name: 'Roadmap.pdf',
          type: 'file',
          ownerId: 'user-001',
          updatedAt: '2026-01-01T00:00:00.000Z',
        },
      ],
    });

    const session = createMemorySessionStore();
    const { service } = createFileServiceStub();
    session.setSession(signedInSession);
    const probe = renderProbe(session, service);
    expect(getDriveCutSelection()).not.toBeNull();

    act(() => {
      session.setSession({
        authToken: 'auth-token-b',
        accessToken: 'access-token-b',
        context: { tenantId: 'tenant-001', userId: 'user-002' },
      });
    });

    expect(probe.getState().scopeKey).toBe('tenant-001:user-002');
    expect(getDriveCutSelection()).toBeNull();
  });
});
