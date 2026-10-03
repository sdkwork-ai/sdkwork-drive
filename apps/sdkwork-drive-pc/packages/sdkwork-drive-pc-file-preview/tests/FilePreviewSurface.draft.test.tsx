/* @vitest-environment jsdom */

import React from 'react';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { FilePreviewSurface } from '../src/components/FilePreviewSurface';
import { DEFAULT_FILE_PREVIEW_LABELS } from '../src/i18n/filePreviewLabels';
import type { FilePreviewResource } from '../src/ports/filePreviewResource';

/**
 * 编辑器的桩：真实实现是 Monaco（数 MB 的编辑器内核，jsdom 装不下）。
 *
 * 桩只保留受控契约——`value` 进来、`onChange` 出去、`onSave` 提交当前值——这正是
 * 本轮修复的核心：草稿归 Surface 持有，编辑器卸载不该丢掉用户写的内容。
 */
vi.mock('../src/components/previews/CodeFileEditor', () => ({
  CodeFileEditor: ({
    onChange,
    onSave,
    value,
  }: {
    onChange: (next: string) => void;
    onSave: (next: string) => void;
    value: string;
  }) => (
    <div>
      <textarea aria-label="editor" value={value} onChange={(event) => onChange(event.target.value)} />
      <button type="button" onClick={() => onSave(value)}>
        editor-save
      </button>
    </div>
  ),
}));

afterEach(() => cleanup());

function createResource(overrides: Partial<FilePreviewResource> = {}): FilePreviewResource {
  return {
    name: 'notes.txt',
    contentType: 'text/plain',
    sizeBytes: 11,
    readText: async () => 'hello world',
    ...overrides,
  };
}

describe('FilePreviewSurface draft handling', () => {
  it('keeps the draft when the operator switches to preview and back', async () => {
    const onDirtyChange = vi.fn();
    render(
      <FilePreviewSurface
        labels={DEFAULT_FILE_PREVIEW_LABELS}
        onDirtyChange={onDirtyChange}
        // 可编辑的前提是宿主给了写回通道。
        resource={createResource({ saveText: async () => undefined })}
      />,
    );

    fireEvent.click(await screen.findByRole('button', { name: DEFAULT_FILE_PREVIEW_LABELS.edit }));
    const editor = await screen.findByLabelText('editor');
    fireEvent.change(editor, { target: { value: 'edited text' } });
    await waitFor(() => expect(onDirtyChange).toHaveBeenLastCalledWith(true));

    // 切到预览：编辑器卸载，但草稿必须还在——显示的是用户写的内容，而不是已保存版本。
    fireEvent.click(screen.getByRole('button', { name: DEFAULT_FILE_PREVIEW_LABELS.preview }));
    expect(await screen.findByText('edited text')).toBeTruthy();
    expect(screen.queryByText('hello world')).toBeNull();
    // 守卫必须保持"脏"，否则关闭时会静默丢内容。
    expect(onDirtyChange).toHaveBeenLastCalledWith(true);

    // 切回编辑：内容仍是草稿。
    fireEvent.click(screen.getByRole('button', { name: DEFAULT_FILE_PREVIEW_LABELS.edit }));
    await waitFor(() => {
      expect((screen.getByLabelText('editor') as HTMLTextAreaElement).value).toBe('edited text');
    });
  });

  it('saves the draft and reports the host so it can refresh its list', async () => {
    const saveText = vi.fn(async () => undefined);
    const onSaved = vi.fn();
    const onDirtyChange = vi.fn();
    render(
      <FilePreviewSurface
        labels={DEFAULT_FILE_PREVIEW_LABELS}
        onDirtyChange={onDirtyChange}
        onSaved={onSaved}
        resource={createResource({ saveText })}
      />,
    );

    fireEvent.click(await screen.findByRole('button', { name: DEFAULT_FILE_PREVIEW_LABELS.edit }));
    fireEvent.change(await screen.findByLabelText('editor'), { target: { value: 'v2' } });
    fireEvent.click(screen.getByRole('button', { name: 'editor-save' }));

    await waitFor(() => expect(saveText).toHaveBeenCalledWith('v2'));
    await waitFor(() => expect(onSaved).toHaveBeenCalledTimes(1));
    // 保存成功后草稿清空，脏状态解除。
    await waitFor(() => expect(onDirtyChange).toHaveBeenLastCalledWith(false));
  });

  it('keeps the draft when the save fails', async () => {
    const saveText = vi.fn(async () => {
      throw new Error('bucket unreachable');
    });
    render(
      <FilePreviewSurface
        labels={DEFAULT_FILE_PREVIEW_LABELS}
        resource={createResource({ saveText })}
      />,
    );

    fireEvent.click(await screen.findByRole('button', { name: DEFAULT_FILE_PREVIEW_LABELS.edit }));
    fireEvent.change(await screen.findByLabelText('editor'), { target: { value: 'v2' } });
    fireEvent.click(screen.getByRole('button', { name: 'editor-save' }));

    // 保存失败必须保留草稿：失败后编辑器里还是用户写的内容，可以直接重试。
    await waitFor(() => expect(saveText).toHaveBeenCalledWith('v2'));
    await waitFor(() => {
      expect((screen.getByLabelText('editor') as HTMLTextAreaElement).value).toBe('v2');
    });
    // 并且不能把状态标成"已保存"。
    expect(screen.queryByText(DEFAULT_FILE_PREVIEW_LABELS.saved)).toBeNull();
  });

  it('reports a failed download instead of failing silently', async () => {
    const download = vi.fn(async () => {
      throw new Error('object exceeds the inline limit');
    });
    render(
      <FilePreviewSurface
        labels={DEFAULT_FILE_PREVIEW_LABELS}
        resource={createResource({ download })}
      />,
    );

    fireEvent.click(await screen.findByRole('button', { name: DEFAULT_FILE_PREVIEW_LABELS.download }));

    await waitFor(() => {
      expect(screen.getByRole('alert').textContent).toContain('object exceeds the inline limit');
    });
  });
});
