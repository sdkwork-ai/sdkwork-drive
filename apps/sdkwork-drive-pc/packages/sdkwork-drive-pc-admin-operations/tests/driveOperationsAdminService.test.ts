import { describe, expect, it, vi } from 'vitest';
import type { DriveBackendSdkClient } from 'sdkwork-drive-pc-admin-core';
import { createSessionStore } from 'sdkwork-drive-pc-core';
import { createDriveOperationsAdminService } from '../src/services/driveOperationsAdminService';

describe('driveOperationsAdminService', () => {
  it('lists audit events through the backend SDK client', async () => {
    const request = vi.fn(async () => ({
      items: [{
        id: 1,
        tenantId: 'tenant-001',
        action: 'drive.maintenance.object_sweep',
        resourceType: 'maintenance_job',
        resourceId: 'job-001',
        operatorId: 'user-001',
        createdAt: '2026-06-25T00:00:00.000Z',
      }],
      pageInfo: {
        mode: 'offset',
        page: 1,
        pageSize: 25,
        totalItems: '1',
      },
    }));
    const backendSdkClient = {
      request,
    } as unknown as DriveBackendSdkClient;
    const session = createSessionStore();
    session.setSession({
      context: {
        tenantId: 'tenant-001',
        userId: 'user-001',
        actorId: 'user-001',
        actorKind: 'user',
      },
    });
    const service = createDriveOperationsAdminService({
      backendSdkClient,
      getSession: session.getSnapshot,
    });

    const page = await service.listAuditEvents({ action: 'drive.maintenance.object_sweep' });

    expect(page.total).toBe(1);
    expect(request).toHaveBeenCalledWith({
      operationId: 'auditEvents.list',
      query: {
        action: 'drive.maintenance.object_sweep',
        resourceType: undefined,
        resourceId: undefined,
        correlationId: undefined,
        traceId: undefined,
        page: undefined,
        page_size: undefined,
      },
      signal: undefined,
    });
  });

  it('starts maintenance sweeps with operator context and upload session timestamp', async () => {
    const request = vi.fn(async () => ({
      scannedCount: 3,
      affectedCount: 1,
      dryRun: true,
    }));
    const backendSdkClient = {
      request,
    } as unknown as DriveBackendSdkClient;
    const session = createSessionStore();
    session.setSession({
      context: {
        tenantId: 'tenant-001',
        userId: 'operator-001',
        actorId: 'operator-001',
        actorKind: 'user',
      },
    });
    const service = createDriveOperationsAdminService({
      backendSdkClient,
      getSession: session.getSnapshot,
    });

    await service.startMaintenanceSweep({
      jobType: 'upload_session_sweep',
      dryRun: true,
      limit: 100,
    });

    expect(request).toHaveBeenCalledWith({
      operationId: 'maintenance.uploadSessionSweep',
      body: expect.objectContaining({
        dryRun: true,
        operatorId: 'operator-001',
        limit: 100,
        nowEpochMs: expect.any(Number),
      }),
    });
  });

  it('loads quota summary through quotas.retrieve', async () => {
    const request = vi.fn(async () => ({
      tenantId: 'tenant-001',
      totalBytes: 4096,
      objectCount: 12,
      quotaBytes: 8192,
    }));
    const backendSdkClient = {
      request,
    } as unknown as DriveBackendSdkClient;
    const service = createDriveOperationsAdminService({
      backendSdkClient,
      getSession: () => ({}),
    });

    const summary = await service.getQuotaSummary();

    expect(summary.totalBytes).toBe(4096);
    expect(summary.quotaBytes).toBe(8192);
    expect(request).toHaveBeenCalledWith({
      operationId: 'quotas.retrieve',
      signal: undefined,
    });
  });

  it('updates quota policy through quotas.update', async () => {
    const request = vi.fn(async () => ({
      tenantId: 'tenant-001',
      totalBytes: 4096,
      objectCount: 12,
      quotaBytes: 1048576,
    }));
    const backendSdkClient = {
      request,
    } as unknown as DriveBackendSdkClient;
    const session = createSessionStore();
    session.setSession({
      context: {
        tenantId: 'tenant-001',
        userId: 'operator-001',
        actorId: 'operator-001',
        actorKind: 'user',
      },
    });
    const service = createDriveOperationsAdminService({
      backendSdkClient,
      getSession: session.getSnapshot,
    });

    const summary = await service.updateQuotaPolicy({ quotaBytes: 1048576 });

    expect(summary.quotaBytes).toBe(1048576);
    expect(request).toHaveBeenCalledWith({
      operationId: 'quotas.update',
      body: {
        quotaBytes: 1048576,
        clearTenantPolicy: undefined,
        operatorId: 'operator-001',
      },
    });
  });

  it('lists labels through labels.list', async () => {
    const request = vi.fn(async () => ({
      items: [{ id: 'label-001', labelKey: 'confidential', displayName: 'Confidential' }],
    }));
    const backendSdkClient = { request } as unknown as DriveBackendSdkClient;
    const service = createDriveOperationsAdminService({
      backendSdkClient,
      getSession: () => ({}),
    });

    const page = await service.listLabels({ lifecycleStatus: 'active' });

    expect(page.items).toHaveLength(1);
    expect(request).toHaveBeenCalledWith({
      operationId: 'labels.list',
      query: {
        lifecycleStatus: 'active',
        page_size: undefined,
        cursor: undefined,
      },
      signal: undefined,
    });
  });

  it('updates labels through labels.update', async () => {
    const request = vi.fn(async () => ({
      id: 'label-001',
      labelKey: 'confidential',
      displayName: 'Restricted',
      color: '#3366FF',
    }));
    const backendSdkClient = { request } as unknown as DriveBackendSdkClient;
    const session = createSessionStore();
    session.setSession({
      context: {
        tenantId: 'tenant-001',
        userId: 'operator-001',
        actorId: 'operator-001',
        actorKind: 'user',
      },
    });
    const service = createDriveOperationsAdminService({
      backendSdkClient,
      getSession: session.getSnapshot,
    });

    const updated = await service.updateLabel('label-001', {
      displayName: 'Restricted',
      color: '#3366FF',
    });

    expect(updated.displayName).toBe('Restricted');
    expect(request).toHaveBeenCalledWith({
      operationId: 'labels.update',
      pathParams: { labelId: 'label-001' },
      body: {
        displayName: 'Restricted',
        color: '#3366FF',
        operatorId: 'operator-001',
      },
    });
  });
});
