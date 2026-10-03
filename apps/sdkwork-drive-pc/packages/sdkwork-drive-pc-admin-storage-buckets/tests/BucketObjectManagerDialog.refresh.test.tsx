/* @vitest-environment jsdom */

import React from 'react';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type {
  StorageProviderAdminService,
  StorageProviderView,
} from 'sdkwork-drive-pc-admin-storage-providers';
import { BucketObjectManagerDialog } from '../src/components/BucketObjectManagerDialog';

/**
 * 预览里保存过内容之后，列表必须重取一次。
 *
 * 真实的保存路径要经过 Monaco（jsdom 装不下编辑器内核），所以这里把预览包换成桩：
 * 桩只暴露两个回调——`onSaved` 与 `onClose`——用来验证宿主这一侧的接线是否成立：
 * 保存 → 关闭 → 列表重新请求。行里的体积与修改时间就是靠这条链路保持新鲜的。
 */
vi.mock('sdkwork-drive-pc-file-preview', async () => {
  const actual = await vi.importActual<typeof import('sdkwork-drive-pc-file-preview')>(
    'sdkwork-drive-pc-file-preview',
  );
  return {
    ...actual,
    FilePreviewSurface: ({
      onClose,
      onSaved,
    }: {
      onClose?: () => void;
      onSaved?: () => void;
    }) => (
      <div>
        <button type="button" onClick={() => onSaved?.()}>
          stub-save
        </button>
        <button type="button" onClick={() => onClose?.()}>
          stub-close
        </button>
      </div>
    ),
  };
});

const MANAGED_BUCKET = 'team-assets';
const PROVIDER = {
  id: 'provider-1',
  displayName: 'MinIO',
  endpointUrl: 'https://minio.internal',
  region: 'cn-north-1',
} as unknown as StorageProviderView;

const OBJECTS = [
  {
    key: 'notes.txt',
    isFolder: false,
    sizeBytes: 11,
    contentType: 'text/plain',
    // 视图模型搬的是 ISO：显示层按宿主语言格式化，服务层不再产出本地化字符串。
    lastModifiedIso: '2026-09-30T10:00:00Z',
  },
];

function createService() {
  const listObjects = vi.fn(
    async (_providerId: string, _input?: { bucket?: string; prefix?: string }) => ({
      items: OBJECTS,
      hasMore: false,
    }),
  );
  const service = {
    listObjects,
    readObjectContent: async () => ({
      content: Buffer.from('hello world', 'utf8').toString('base64'),
      contentType: 'text/plain',
    }),
    writeObjectContent: async () => OBJECTS[0],
  } as unknown as StorageProviderAdminService;
  return { listObjects, service };
}

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe('BucketObjectManagerDialog list freshness', () => {
  it('reloads the list when a preview save is followed by closing the preview', async () => {
    const { listObjects, service } = createService();
    render(
      <BucketObjectManagerDialog
        bucket={MANAGED_BUCKET}
        onOpenChange={() => undefined}
        open
        provider={PROVIDER}
        service={service}
      />,
    );

    expect(await screen.findByText('notes.txt')).toBeTruthy();
    const initialCalls = listObjects.mock.calls.length;

    // 打开预览 → 保存 → 关闭：宿主应当重取当前目录。
    // 没有 i18n provider 时 `t` 回落成 key，所以按钮名就是词条名。
    fireEvent.click(screen.getAllByRole('button', { name: 'bucketsPreviewFile' })[0]);
    fireEvent.click(await screen.findByRole('button', { name: 'stub-save' }));
    fireEvent.click(screen.getByRole('button', { name: 'stub-close' }));

    await waitFor(() => {
      expect(listObjects.mock.calls.length).toBeGreaterThan(initialCalls);
    });
    expect(listObjects.mock.calls.at(-1)?.[1]?.bucket).toBe(MANAGED_BUCKET);
  });

  it('does not reload when the preview is closed without saving', async () => {
    const { listObjects, service } = createService();
    render(
      <BucketObjectManagerDialog
        bucket={MANAGED_BUCKET}
        onOpenChange={() => undefined}
        open
        provider={PROVIDER}
        service={service}
      />,
    );

    expect(await screen.findByText('notes.txt')).toBeTruthy();
    const initialCalls = listObjects.mock.calls.length;

    fireEvent.click(screen.getAllByRole('button', { name: 'bucketsPreviewFile' })[0]);
    fireEvent.click(await screen.findByRole('button', { name: 'stub-close' }));

    // 没改过内容就不该多打一次列表请求：翻看预览是高频操作。
    await waitFor(() => {
      expect(screen.queryByRole('button', { name: 'stub-close' })).toBeNull();
    });
    expect(listObjects.mock.calls.length).toBe(initialCalls);
  });
});
