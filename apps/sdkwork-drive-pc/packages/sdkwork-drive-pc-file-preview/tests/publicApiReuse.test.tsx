/* @vitest-environment jsdom */

import React from 'react';
import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import {
  ArchivePreview,
  AudioFilePreview,
  CodeFilePreview,
  CodeFileEditor,
  FileKindIcon,
  FilePreviewSurface,
  ImageFilePreview,
  PdfFilePreview,
  PresentationPreviewError,
  PreviewErrorBoundary,
  SlidesPreview,
  SpreadsheetPreview,
  TextFilePreview,
  UnsupportedFilePreview,
  VideoFilePreview,
  WordDocumentPreview,
  WordDocumentPreviewError,
  ZipArchiveError,
  FILE_PREVIEW_KINDS,
  mergeFilePreviewLabels,
  readZipArchive,
} from '../src/index';
import type { FilePreviewLabels, FilePreviewResource } from '../src/index';
import { buildZipArchive } from './ooxml/zipFixture';

/**
 * 「第二个宿主」证明：只依赖本包的公开出口，能不能把每个组件挂起来。
 *
 * 这个文件刻意不 import 任何 sdkwork-drive-pc-commons / admin-* / SDK 包，标签与内容通道
 * 都在这里从零实现——它代表另一个控制台（例如 drive 控制台）接入时要写的那几十行。若
 * 哪天有人把宿主概念塞回共享包，这个文件会先失败。
 *
 * Monaco 用桩替换真实引擎（数 MB 的编辑器内核，jsdom 跑不动：连
 * `document.queryCommandSupported` 都没有），但桩保留 `@monaco-editor/react` 的对外契约
 * （value / onChange / language），所以代码预览与编辑器这两条最"重"的路径也真的被渲染到了，
 * 而不是只断言"导出的是个组件"。
 */
vi.mock('@monaco-editor/react', () => ({
  loader: { config: vi.fn() },
  default: ({
    language,
    onChange,
    value,
  }: {
    language?: string;
    onChange?: (next: string) => void;
    value?: string;
  }) => (
    <textarea
      aria-label="monaco-stub"
      data-language={language}
      value={value ?? ''}
      onChange={(event) => onChange?.(event.target.value)}
    />
  ),
}));

// `setupMonaco` 会读一次语言注册表来校验语言 id；真实的 monaco-editor 在 jsdom 里
// 会在导入期就假设浏览器 API 存在，所以这里给出最小替身。
vi.mock('monaco-editor', () => ({
  languages: { getLanguages: () => [{ id: 'typescript' }, { id: 'plaintext' }] },
}));
const labels: FilePreviewLabels = mergeFilePreviewLabels({
  // 只覆盖两个字段，其余走包的英文兜底：宿主不必翻译整套词条。
  download: 'Save a copy',
  kind: {
    folder: 'Folder',
    image: 'Image',
    video: 'Video',
    audio: 'Audio',
    pdf: 'PDF',
    word: 'Document',
    spreadsheet: 'Spreadsheet',
    presentation: 'Slides',
    archive: 'Archive',
    text: 'Text',
    markdown: 'Markdown',
    code: 'Code',
    unsupported: 'File',
  },
});

/** 第二个宿主自己的内容通道：这里用内存数据，真实宿主换成自己的 SDK 调用即可。 */
function createHostResource(overrides: Partial<FilePreviewResource> = {}): FilePreviewResource {
  return {
    name: 'readme.txt',
    contentType: 'text/plain',
    sizeBytes: 12,
    readText: async () => 'host-provided text',
    ...overrides,
  };
}

const ZIP = buildZipArchive([
  { path: 'docs/', directory: true },
  { path: 'docs/a.txt', text: 'hello' },
]);

beforeAll(() => {
  if (typeof URL.createObjectURL !== 'function') {
    Object.defineProperty(URL, 'createObjectURL', { configurable: true, value: () => 'blob:host' });
    Object.defineProperty(URL, 'revokeObjectURL', { configurable: true, value: () => undefined });
  }
});

afterEach(() => cleanup());

describe('public API reuse by a second host', () => {
  it('mounts every preview component with only the public exports', async () => {
    const media = { bytes: new Uint8Array([1, 2, 3]), contentType: 'image/png' };

    const { unmount: unmountImage } = render(
      <ImageFilePreview labels={labels} name="photo.png" source={media} />,
    );
    expect(screen.getByText('photo.png')).toBeTruthy();
    unmountImage();

    const { unmount: unmountVideo } = render(
      <VideoFilePreview labels={labels} name="clip.mp4" source={media} />,
    );
    expect(screen.getByText('clip.mp4')).toBeTruthy();
    unmountVideo();

    const { unmount: unmountAudio } = render(
      <AudioFilePreview labels={labels} name="song.mp3" source={media} />,
    );
    expect(screen.getByText('song.mp3')).toBeTruthy();
    unmountAudio();

    const { unmount: unmountPdf } = render(
      <PdfFilePreview labels={labels} name="manual.pdf" source={media} />,
    );
    expect(screen.getByText('manual.pdf')).toBeTruthy();
    unmountPdf();

    render(<TextFilePreview labels={labels} name="notes.txt" text="line one" />);
    expect(screen.getByText(/line one/)).toBeTruthy();
    cleanup();

    // Office 与压缩包预览没有可见头部（种类徽标在 Surface 工具条上），所以这里断言
    // "挂载成功且渲染出内容"，而不是某个具体词条：本用例要证明的是无需宿主依赖即可渲染。
    for (const element of [
      <WordDocumentPreview bytes={ZIP} labels={labels} name="doc.docx" key="word" />,
      <SlidesPreview bytes={ZIP} labels={labels} name="deck.pptx" key="slides" />,
      <SpreadsheetPreview
        bytes={ZIP}
        contentType="text/csv"
        labels={labels}
        name="sheet.csv"
        key="sheet"
      />,
    ]) {
      const { container, unmount } = render(element);
      expect((container.textContent ?? '').trim().length).toBeGreaterThan(0);
      unmount();
    }

    render(<ArchivePreview bytes={ZIP} labels={labels} name="bundle.zip" />);
    expect(screen.getByText(/docs\/a\.txt/)).toBeTruthy();
    cleanup();

    render(
      <UnsupportedFilePreview
        contentType="application/x-7z-compressed"
        labels={labels}
        name="backup.7z"
        sizeBytes={1024}
      />,
    );
    expect(screen.getByText('backup.7z')).toBeTruthy();
    cleanup();

    render(<FileKindIcon kind="folder" />);
    expect(document.querySelector('svg')).toBeTruthy();
    cleanup();

    // 编辑器与代码预览：Monaco 已用桩替换，所以这里真的渲染（Suspense 需要等待懒加载）。
    const { unmount: unmountCodePreview } = render(
      <React.Suspense fallback={<p>loading</p>}>
        <CodeFilePreview
          labels={labels}
          name="app.ts"
          text={'export const answer = 42;\n'}
        />
      </React.Suspense>,
    );
    expect(await screen.findByLabelText('monaco-stub')).toBeTruthy();
    unmountCodePreview();

    const { unmount: unmountEditor } = render(
      <React.Suspense fallback={<p>loading</p>}>
        <CodeFileEditor
          labels={labels}
          name="app.ts"
          value={'export const answer = 42;\n'}
          onChange={() => undefined}
          onSave={() => undefined}
        />
      </React.Suspense>,
    );
    const editor = await screen.findByLabelText('monaco-stub');
    expect((editor as HTMLTextAreaElement).value).toContain('answer = 42');
    unmountEditor();
  });

  it('drives the surface through the port with a host-provided label subset', async () => {
    render(
      <FilePreviewSurface
        labels={labels}
        resource={createHostResource({ download: () => undefined })}
      />,
    );

    expect(await screen.findByText('host-provided text')).toBeTruthy();
    // 宿主只覆盖了 download，其余字段由包的英文兜底补齐（不会漏出词条键）。
    expect(screen.getByRole('button', { name: 'Save a copy' })).toBeTruthy();
    expect(screen.getByText('Text')).toBeTruthy();
  });

  it('exposes a container check the host can use before parsing', () => {
    // 第二个宿主可以自己做容器判定，不必知道 OOXML 细节。
    expect(readZipArchive(ZIP).byPath.has('docs/a.txt')).toBe(true);
  });

  it('keeps the error types and fallback boundary importable', () => {
    expect(new ZipArchiveError('not-a-zip', 'x').code).toBe('not-a-zip');
    expect(new WordDocumentPreviewError('document-xml-invalid', 'x').code).toBe(
      'document-xml-invalid',
    );
    expect(new PresentationPreviewError('dom-parser-unavailable', 'x').code).toBe(
      'dom-parser-unavailable',
    );
    render(
      <PreviewErrorBoundary fallback={() => <p>chunk failed</p>}>
        <p>ok</p>
      </PreviewErrorBoundary>,
    );
    expect(screen.getByText('ok')).toBeTruthy();
  });

  it('publishes a runtime list of kinds so a host can enumerate them', () => {
    expect(FILE_PREVIEW_KINDS).toContain('folder');
    expect(new Set(FILE_PREVIEW_KINDS).size).toBe(FILE_PREVIEW_KINDS.length);
  });
});
