/* @vitest-environment jsdom */

import React from 'react';
import { cleanup, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { FilePreviewSurface } from '../src/components/FilePreviewSurface';
import { DEFAULT_FILE_PREVIEW_LABELS } from '../src/i18n/filePreviewLabels';
import type { FilePreviewResource } from '../src/ports/filePreviewResource';
import { buildZipArchive } from './ooxml/zipFixture';

afterEach(() => cleanup());

// jsdom 没有实现 object URL；图片/媒体预览用它把字节变成可寻址 URL。
beforeAll(() => {
  if (typeof URL.createObjectURL !== 'function') {
    Object.defineProperty(URL, 'createObjectURL', { configurable: true, value: () => 'blob:test' });
    Object.defineProperty(URL, 'revokeObjectURL', { configurable: true, value: () => undefined });
  }
});

/** 非 ZIP 的字节：旧版二进制 Office（OLE 复合文档魔数）就长这样。 */
const LEGACY_OFFICE_BYTES = new Uint8Array([0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]);

/** 合法但空的 ZIP：只有 End of Central Directory 记录。 */
function createEmptyZipArchive(): Uint8Array {
  const bytes = new Uint8Array(22);
  bytes.set([0x50, 0x4b, 0x05, 0x06], 0);
  return bytes;
}

function createResource(overrides: Partial<FilePreviewResource> = {}): FilePreviewResource {
  return {
    name: 'notes.txt',
    contentType: 'text/plain',
    sizeBytes: 11,
    readText: async () => 'hello world',
    ...overrides,
  };
}

describe('FilePreviewSurface', () => {
  it('renders the text preview for a text resource', async () => {
    render(<FilePreviewSurface labels={DEFAULT_FILE_PREVIEW_LABELS} resource={createResource()} />);

    expect(await screen.findByText('hello world')).toBeTruthy();
    expect(screen.getByText(DEFAULT_FILE_PREVIEW_LABELS.kind.text)).toBeTruthy();
  });

  it('never reads a file that exceeds the inline limit', async () => {
    const readBytes = vi.fn(async () => new Uint8Array());
    render(
      <FilePreviewSurface
        labels={DEFAULT_FILE_PREVIEW_LABELS}
        resource={createResource({
          name: 'archive.zip',
          contentType: 'application/zip',
          sizeBytes: 64 * 1024 * 1024,
          inlineLimitBytes: 8 * 1024 * 1024,
          readBytes,
          readText: undefined,
        })}
      />,
    );

    expect(await screen.findByText(DEFAULT_FILE_PREVIEW_LABELS.tooLargeTitle)).toBeTruthy();
    // 超限的文件连一次内容请求都不该发出去。
    expect(readBytes).not.toHaveBeenCalled();
  });

  it('degrades a legacy binary Office file to the honest fallback panel', async () => {
    render(
      <FilePreviewSurface
        labels={DEFAULT_FILE_PREVIEW_LABELS}
        resource={createResource({
          name: 'contract.doc',
          contentType: 'application/msword',
          readBytes: async () => LEGACY_OFFICE_BYTES,
          readText: undefined,
        })}
      />,
    );

    // 没有可用的 doc 解析器：给出「下载后查看」而不是一句解析报错。
    expect(await screen.findByText(DEFAULT_FILE_PREVIEW_LABELS.unavailableHint)).toBeTruthy();
    // 文件名同时出现在工具条与兜底面板里。
    expect(screen.getAllByText('contract.doc').length).toBeGreaterThan(0);
  });

  it('treats a ZIP that is not the Word package as a different format', async () => {
    // 合法的空 ZIP：读得出来，但没有 `word/document.xml`。`.odt`、Pages 文档以及结构
    // 损坏的 docx 都是这个样子——必须落到兜底面板，而不是对非空文件说「内容为空」。
    render(
      <FilePreviewSurface
        labels={DEFAULT_FILE_PREVIEW_LABELS}
        resource={createResource({
          name: 'contract.docx',
          contentType:
            'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
          readBytes: async () => createEmptyZipArchive(),
          readText: undefined,
        })}
      />,
    );

    expect(await screen.findByText(DEFAULT_FILE_PREVIEW_LABELS.unavailableHint)).toBeTruthy();
    expect(screen.queryByText(DEFAULT_FILE_PREVIEW_LABELS.emptyFile)).toBeNull();
  });

  it('routes a ZIP that does contain the Word part to the Word preview', async () => {
    // 同一份容器里放上标记部件：判定必须放行，由 Word 解析器自己去报「内容读不出来」。
    const archive = buildZipArchive([{ path: 'word/document.xml', text: '<w:document>' }]);
    render(
      <FilePreviewSurface
        labels={DEFAULT_FILE_PREVIEW_LABELS}
        resource={createResource({
          name: 'contract.docx',
          contentType:
            'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
          readBytes: async () => archive,
          readText: undefined,
        })}
      />,
    );

    await waitFor(() => {
      expect(screen.queryByText(DEFAULT_FILE_PREVIEW_LABELS.unavailableHint)).toBeNull();
    });
  });

  it('reports the resolved preview kind in the header', async () => {
    render(
      <FilePreviewSurface
        labels={DEFAULT_FILE_PREVIEW_LABELS}
        resource={createResource({
          name: 'photo.png',
          contentType: 'image/png',
          readText: undefined,
          readBytes: async () => new Uint8Array([0x89, 0x50, 0x4e, 0x47]),
        })}
      />,
    );

    expect(await screen.findByText(DEFAULT_FILE_PREVIEW_LABELS.kind.image)).toBeTruthy();
  });

  it('surfaces a read failure with a retry action', async () => {
    let attempt = 0;
    render(
      <FilePreviewSurface
        labels={DEFAULT_FILE_PREVIEW_LABELS}
        resource={createResource({
          readText: async () => {
            attempt += 1;
            if (attempt === 1) {
              throw new Error('bucket unreachable');
            }
            return 'recovered';
          },
        })}
      />,
    );

    expect(await screen.findByText('bucket unreachable')).toBeTruthy();
    screen.getByRole('button', { name: DEFAULT_FILE_PREVIEW_LABELS.retry }).click();
    expect(await screen.findByText('recovered')).toBeTruthy();
  });
});
