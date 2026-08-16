import { describe, expect, it, vi } from 'vitest';
import {
  createDriveUploaderClient,
  type DriveUploaderTransport,
} from '@sdkwork/drive-app-sdk';

const AMBIENT_CALLER_IDENTITY_FIELDS = [
  'tenantId',
  'organizationId',
  'userId',
  'anonymousId',
  'operatorId',
  'appId',
] as const;

function expectBusinessOnlyRequest(body: object): void {
  for (const field of AMBIENT_CALLER_IDENTITY_FIELDS) {
    expect(body).not.toHaveProperty(field);
  }
}

function createReplacementTransport(): {
  transport: DriveUploaderTransport;
  calls: string[];
} {
  const calls: string[] = [];
  const transport: DriveUploaderTransport = {
    drive: {
      uploader: {
        uploads: {
          create: vi.fn(),
          parts: {
            update: vi.fn(),
          },
        },
      },
      uploadSessions: {
        create: vi.fn(async (body) => {
          calls.push('uploadSessions.create');
          expectBusinessOnlyRequest(body);
          return {
            id: body.sessionId,
            tenantId: 'tenant-001',
            spaceId: body.spaceId,
            nodeId: body.nodeId,
            bucket: 'bucket-001',
            objectKey: 'objects/replacement',
            idempotencyKey: body.idempotencyKey,
            state: 'created' as const,
            expiresAtEpochMs: body.expiresAtEpochMs,
            version: '1',
            storageProviderId: 'provider-001',
            storageUploadId: 'storage-upload-replacement',
          };
        }),
        parts: {
          update: vi.fn(async (_uploadSessionId, partNo) => {
            calls.push('uploadSessions.parts.update');
            return {
              uploadUrl: `https://storage.example.test/replacement/${partNo}`,
              method: 'PUT' as const,
              headers: {
                'x-sdkwork-drive': 'replacement',
              },
              partNo,
              uploadId: 'storage-upload-replacement',
              expiresAtEpochMs: '1800000000000',
            };
          }),
        },
        complete: vi.fn(async (uploadSessionId, body) => {
          calls.push('uploadSessions.complete');
          expectBusinessOnlyRequest(body);
          return {
            id: uploadSessionId,
            tenantId: 'tenant-001',
            spaceId: 'my-storage',
            nodeId: 'file-001',
            bucket: 'bucket-001',
            objectKey: 'objects/replacement',
            state: 'completed' as const,
            expiresAtEpochMs: '1800000000000',
            version: '2',
            storageProviderId: 'provider-001',
            storageUploadId: body.uploadId || 'storage-upload-replacement',
          };
        }),
        abort: vi.fn(async (uploadSessionId, body) => {
          calls.push('uploadSessions.abort');
          expectBusinessOnlyRequest(body);
          return {
            id: uploadSessionId,
            tenantId: 'tenant-001',
            spaceId: 'my-storage',
            nodeId: 'file-001',
            bucket: 'bucket-001',
            objectKey: 'objects/replacement',
            state: 'aborted' as const,
            expiresAtEpochMs: '1800000000000',
            version: '1',
            storageProviderId: 'provider-001',
            storageUploadId: 'storage-upload-replacement',
          };
        }),
      },
    },
  };
  return { transport, calls };
}

describe('Drive uploader composed client contract', () => {
  it('reuses uploaded parts from prior attempts when the upload session is stable', async () => {
    const calls: string[] = [];
    const transport: DriveUploaderTransport = {
      drive: {
        uploader: {
          uploads: {
            create: vi.fn(async (body) => {
              calls.push('uploader.uploads.create');
              expectBusinessOnlyRequest(body);
              return {
                uploadItem: {
                  id: body.nowEpochMs === '1700000000000' ? 'upload-item-a' : 'upload-item-b',
                  taskId: body.taskId,
                  tenantId: 'tenant-001',
                  userId: 'user-001',
                  actorType: 'user',
                  actorId: 'actor-001',
                  appId: 'drive-pc',
                  appResourceType: body.appResourceType,
                  appResourceId: body.appResourceId,
                  scene: body.scene,
                  source: body.source,
                  uploadProfileCode: body.uploadProfileCode,
                  contentTypeGroup: 'document',
                  fileFingerprint: body.fileFingerprint,
                  spaceId: body.spaceId,
                  nodeId: 'file-001',
                  uploadSessionId: 'upload-stable',
                  storageUploadId: 'storage-upload-stable',
                  originalFileName: body.originalFileName,
                  contentType: body.contentType,
                  contentLength: body.contentLength,
                  chunkSizeBytes: body.chunkSizeBytes,
                  totalParts: '2',
                  uploadedPartsCount: '0',
                  uploadedBytes: '0',
                  status: 'prepared',
                  retentionMode: 'long_term',
                  cleanupStatus: 'active',
                  postProcessStatus: 'not_required',
                },
                uploadSession: {
                  id: 'upload-stable',
                  tenantId: 'tenant-001',
                  spaceId: body.spaceId,
                  nodeId: 'file-001',
                  bucket: 'bucket-001',
                  objectKey: 'objects/upload-stable',
                  state: 'created' as const,
                  storageProviderId: 'provider-001',
                  storageUploadId: 'storage-upload-stable',
                  expiresAtEpochMs: '1800000000000',
                  version: '1',
                },
              };
            }),
            parts: {
              update: vi.fn(async (_uploadItemId, partNo) => {
                calls.push(`uploader.uploads.parts.update:${partNo}`);
                return { id: `part-${partNo}`, status: 'uploaded' } as any;
              }),
            },
          },
        },
        uploadSessions: {
          create: vi.fn() as any,
          parts: {
            update: vi.fn(async (_uploadSessionId, partNo) => {
              calls.push(`uploadSessions.parts.update:${partNo}`);
              return {
                uploadUrl: `https://storage.example.test/upload/${partNo}`,
                method: 'PUT' as const,
                headers: {},
                partNo,
                uploadId: 'storage-upload-stable',
                expiresAtEpochMs: '1800000000000',
              };
            }),
          },
          complete: vi.fn(async () => {
            calls.push('complete');
            return { id: 'upload-stable', state: 'completed' } as any;
          }),
          abort: vi.fn(async () => {
            calls.push('abort');
            return { id: 'upload-stable', state: 'aborted' } as any;
          }),
        },
      },
    };

    const uploadedPartsByAttempt: Array<number[]> = [];
    const uploadFetch = vi.fn<typeof fetch>(async (_url, init) => {
      const body = init?.body as Blob;
      uploadedPartsByAttempt.at(-1)?.push(body.size);
      return new Response('', { status: 200, headers: { ETag: `"etag-${body.size}"` } });
    });

    const stateStore = {
      snapshot: undefined as any,
      async get() {
        return this.snapshot;
      },
      async put(snapshot: any) {
        this.snapshot = snapshot;
      },
      async clear() {
        this.snapshot = undefined;
      },
    };
    const client = createDriveUploaderClient({ transport, uploadFetch, stateStore: stateStore as any, defaultChunkSizeBytes: 5 });

    // Attempt 1: fail after part 1 uploaded.
    uploadedPartsByAttempt.push([]);
    let attempt1Call = 0;
    uploadFetch.mockImplementation(async (_url, init) => {
      const body = init?.body as Blob;
      uploadedPartsByAttempt.at(-1)?.push(body.size);
      attempt1Call += 1;
      return attempt1Call === 1
        ? new Response('', { status: 200, headers: { ETag: '"etag-1"' } })
        : new Response('', { status: 503 });
    });
    await expect(client.upload({
      file: new File([new Uint8Array(10)], 'split.txt', { type: 'text/plain' }),
      appResourceType: 'desktop-file-browser',
      appResourceId: 'my-storage',
      spaceId: 'my-storage',
      nowEpochMs: '1700000000000',
    })).rejects.toThrow(/503/);

    // Attempt 2: should skip part 1 and only upload part 2.
    uploadedPartsByAttempt.push([]);
    uploadFetch.mockImplementation(async (_url, init) => {
      const body = init?.body as Blob;
      uploadedPartsByAttempt.at(-1)?.push(body.size);
      return new Response('', { status: 200, headers: { ETag: `"etag-${body.size}"` } });
    });
    await client.upload({
      file: new File([new Uint8Array(10)], 'split.txt', { type: 'text/plain' }),
      appResourceType: 'desktop-file-browser',
      appResourceId: 'my-storage',
      spaceId: 'my-storage',
      nowEpochMs: '1700000000001',
    });

    expect(uploadedPartsByAttempt[0]).toEqual([5, 5]);
    expect(uploadedPartsByAttempt[1]).toEqual([5]);
    expect(calls).toContain('complete');
  });

  it('replaces existing node content inside the composed SDK boundary', async () => {
    const { transport, calls } = createReplacementTransport();
    const uploadFetch = vi.fn<typeof fetch>(async () =>
      new Response('', {
        status: 200,
        headers: {
          ETag: '"etag-replacement"',
        },
      }),
    );
    const client = createDriveUploaderClient({ transport, uploadFetch });

    const result = await client.replaceNodeContent({
      file: new File(['# Updated\n'], 'Roadmap.md', { type: 'text/markdown' }),
      spaceId: 'my-storage',
      nodeId: 'file-001',
      appResourceType: 'desktop-file-editor',
      appResourceId: 'file-001',
      scene: 'drive_pc_text_save',
      source: 'pc_text_editor',
      uploadProfileCode: 'text',
      originalFileName: 'Roadmap.md',
      contentType: 'text/markdown',
      nowEpochMs: '1800000000000',
    });

    expect(calls).toEqual([
      'uploadSessions.create',
      'uploadSessions.parts.update',
      'uploadSessions.complete',
    ]);
    expect(transport.drive.uploadSessions.create).toHaveBeenCalledWith(expect.objectContaining({
      spaceId: 'my-storage',
      nodeId: 'file-001',
    }), expect.any(Object));
    expect(uploadFetch).toHaveBeenCalledWith(
      'https://storage.example.test/replacement/1',
      expect.objectContaining({
        method: 'PUT',
        headers: {
          'x-sdkwork-drive': 'replacement',
        },
        body: expect.any(Blob),
      }),
    );
    expect(transport.drive.uploadSessions.complete).toHaveBeenCalledWith(
      expect.any(String),
      expect.objectContaining({
        uploadId: 'storage-upload-replacement',
        contentType: 'text/markdown',
        contentLength: '10',
        checksumSha256Hex: 'sha256:ff5ae8bf981b37541f866f85b80010440b7349a1bc57cd83c79c0b2be12cc04b',
        parts: [
          {
            partNo: 1,
            etag: '"etag-replacement"',
          },
        ],
      }),
      expect.any(Object),
    );
    expect(result.uploadSession.state).toBe('completed');
    expect(result.parts).toEqual([
      {
        partNo: 1,
        etag: '"etag-replacement"',
        offsetBytes: 0,
        sizeBytes: 10,
      },
    ]);
  });

  it('aborts replacement sessions inside the composed SDK when provider upload fails', async () => {
    const { transport, calls } = createReplacementTransport();
    const client = createDriveUploaderClient({
      transport,
      uploadFetch: vi.fn<typeof fetch>(async () => new Response('', { status: 503 })),
    });

    await expect(client.replaceNodeContent({
      file: new File(['# Updated\n'], 'Roadmap.md', { type: 'text/markdown' }),
      spaceId: 'my-storage',
      nodeId: 'file-001',
      appResourceType: 'desktop-file-editor',
      appResourceId: 'file-001',
      originalFileName: 'Roadmap.md',
      contentType: 'text/markdown',
      nowEpochMs: '1800000000000',
    })).rejects.toThrow('Drive uploader signed upload failed with HTTP 503.');

    expect(calls).toEqual([
      'uploadSessions.create',
      'uploadSessions.parts.update',
      'uploadSessions.abort',
    ]);
    expect(transport.drive.uploadSessions.abort).toHaveBeenCalledWith(
      expect.any(String),
      {},
      expect.any(Object),
    );
  });

  it('aborts prepared upload sessions inside the composed SDK when provider upload fails', async () => {
    const { transport, calls } = createReplacementTransport();
    transport.drive.uploader.uploads.create = vi.fn(async (body) => {
      calls.push('uploader.uploads.create');
      expectBusinessOnlyRequest(body);
      return {
        uploadItem: {
          id: body.id || 'upload-item-failed',
          taskId: body.taskId || 'task-failed',
          tenantId: 'tenant-001',
          userId: 'user-001',
          actorType: 'user',
          actorId: 'actor-001',
          appId: 'drive-pc',
          appResourceType: body.appResourceType,
          appResourceId: body.appResourceId,
          scene: body.scene,
          source: body.source,
          uploadProfileCode: body.uploadProfileCode,
          contentTypeGroup: 'document',
          fileFingerprint: body.fileFingerprint,
          spaceId: body.spaceId,
          nodeId: 'file-failed',
          uploadSessionId: 'upload-failed',
          storageUploadId: 'storage-upload-failed',
          originalFileName: body.originalFileName,
          contentType: body.contentType,
          contentLength: body.contentLength,
          chunkSizeBytes: body.chunkSizeBytes,
          totalParts: '1',
          uploadedPartsCount: '0',
          uploadedBytes: '0',
          status: 'prepared',
          retentionMode: 'long_term',
          cleanupStatus: 'active',
          postProcessStatus: 'not_required',
        },
        uploadSession: {
          id: 'upload-failed',
          tenantId: 'tenant-001',
          spaceId: body.spaceId,
          nodeId: 'file-failed',
          bucket: 'bucket-001',
          objectKey: 'objects/upload-failed',
          state: 'created' as const,
          storageProviderId: 'provider-001',
          storageUploadId: 'storage-upload-failed',
          expiresAtEpochMs: '1800000000000',
          version: '1',
        },
      };
    });
    transport.drive.uploadSessions.parts.update = vi.fn(async (_uploadSessionId, partNo) => {
      calls.push('uploadSessions.parts.update');
      return {
        uploadUrl: `https://storage.example.test/upload-failed/${partNo}`,
        method: 'PUT' as const,
        headers: {},
        partNo,
        uploadId: 'storage-upload-failed',
        expiresAtEpochMs: '1800000000000',
      };
    });
    const client = createDriveUploaderClient({
      transport,
      uploadFetch: vi.fn<typeof fetch>(async () => new Response('', { status: 503 })),
    });

    await expect(client.upload({
      file: new File(['hello'], 'Roadmap.pdf', { type: 'application/pdf' }),
      appResourceType: 'desktop-file-browser',
      appResourceId: 'my-storage',
      scene: 'drive_pc_file_upload',
      source: 'pc_local_file',
      originalFileName: 'Roadmap.pdf',
      contentType: 'application/pdf',
      spaceId: 'my-storage',
      nowEpochMs: '1800000000000',
    })).rejects.toThrow('Drive uploader signed upload failed with HTTP 503.');

    expect(calls).toEqual([
      'uploader.uploads.create',
      'uploadSessions.parts.update',
    ]);
    expect(transport.drive.uploadSessions.abort).not.toHaveBeenCalled();
  });

  it('plans multipart replacement uploads inside the composed SDK boundary', async () => {
    const { transport, calls } = createReplacementTransport();
    const uploadedBodies: string[] = [];
    const client = createDriveUploaderClient({
      transport,
      defaultChunkSizeBytes: 5,
      uploadFetch: vi.fn<typeof fetch>(async (url, init) => {
        const body = init?.body as Blob;
        uploadedBodies.push(new TextDecoder().decode(await body.arrayBuffer()));
        return new Response('', {
          status: 200,
          headers: {
            ETag: `"etag-${String(url).slice(-1)}"`,
          },
        });
      }),
    });

    const result = await client.replaceNodeContent({
      file: new File(['0123456789'], 'split.txt', { type: 'text/plain' }),
      spaceId: 'my-storage',
      nodeId: 'file-001',
      appResourceType: 'desktop-file-editor',
      appResourceId: 'file-001',
      originalFileName: 'split.txt',
      contentType: 'text/plain',
      nowEpochMs: '1800000000000',
    });

    expect(calls).toEqual([
      'uploadSessions.create',
      'uploadSessions.parts.update',
      'uploadSessions.parts.update',
      'uploadSessions.complete',
    ]);
    expect(uploadedBodies).toEqual(['01234', '56789']);
    expect(transport.drive.uploadSessions.parts.update).toHaveBeenNthCalledWith(
      1,
      expect.any(String),
      1,
      expect.any(Object),
      expect.any(Object),
    );
    expect(transport.drive.uploadSessions.parts.update).toHaveBeenNthCalledWith(
      2,
      expect.any(String),
      2,
      expect.any(Object),
      expect.any(Object),
    );
    expect(result.parts).toEqual([
      {
        partNo: 1,
        etag: '"etag-1"',
        offsetBytes: 0,
        sizeBytes: 5,
      },
      {
        partNo: 2,
        etag: '"etag-2"',
        offsetBytes: 5,
        sizeBytes: 5,
      },
    ]);
    expect(transport.drive.uploadSessions.complete).toHaveBeenCalledWith(
      expect.any(String),
      expect.objectContaining({
        parts: [
          {
            partNo: 1,
            etag: '"etag-1"',
          },
          {
            partNo: 2,
            etag: '"etag-2"',
          },
        ],
      }),
      expect.any(Object),
    );
  });
});
