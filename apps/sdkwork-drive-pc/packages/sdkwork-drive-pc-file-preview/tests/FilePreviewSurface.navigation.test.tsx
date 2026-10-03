/* @vitest-environment jsdom */

import React from 'react';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { FilePreviewSurface } from '../src/components/FilePreviewSurface';
import { DEFAULT_FILE_PREVIEW_LABELS } from '../src/i18n/filePreviewLabels';
import type { FilePreviewResource } from '../src/ports/filePreviewResource';

/**
 * 相邻文件切换。
 *
 * 序列由宿主给出（它才知道目录内容与当前筛选），预览层只负责"上一处/下一处 + 位置"，
 * 并且必须给输入框和播放器让路：方向键在这两处各有主人。
 */
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

describe('FilePreviewSurface navigation', () => {
  it('renders the position and switches with the buttons', async () => {
    const onNext = vi.fn();
    const onPrevious = vi.fn();
    render(
      <FilePreviewSurface
        labels={DEFAULT_FILE_PREVIEW_LABELS}
        navigation={{ index: 1, onNext, onPrevious, total: 5 }}
        resource={createResource()}
      />,
    );

    expect(await screen.findByText('2 / 5')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: DEFAULT_FILE_PREVIEW_LABELS.nextFile }));
    fireEvent.click(screen.getByRole('button', { name: DEFAULT_FILE_PREVIEW_LABELS.previousFile }));

    expect(onNext).toHaveBeenCalledTimes(1);
    expect(onPrevious).toHaveBeenCalledTimes(1);
  });

  it('switches with the arrow keys', async () => {
    const onNext = vi.fn();
    const onPrevious = vi.fn();
    const { container } = render(
      <FilePreviewSurface
        labels={DEFAULT_FILE_PREVIEW_LABELS}
        navigation={{ index: 1, onNext, onPrevious, total: 5 }}
        resource={createResource()}
      />,
    );

    const surface = container.querySelector('section') as HTMLElement;
    fireEvent.keyDown(surface, { key: 'ArrowRight' });
    fireEvent.keyDown(surface, { key: 'ArrowLeft' });

    expect(onNext).toHaveBeenCalledTimes(1);
    expect(onPrevious).toHaveBeenCalledTimes(1);
  });

  it('disables the ends instead of wrapping around', async () => {
    const onNext = vi.fn();
    render(
      <FilePreviewSurface
        labels={DEFAULT_FILE_PREVIEW_LABELS}
        navigation={{ index: 0, onNext, total: 3 }}
        resource={createResource()}
      />,
    );

    const previous = await screen.findByRole('button', {
      name: DEFAULT_FILE_PREVIEW_LABELS.previousFile,
    });
    expect((previous as HTMLButtonElement).disabled).toBe(true);
    // 首个文件再按左键不应触发任何事（也没有可触发的回调）。
    fireEvent.keyDown(previous.closest('section') as HTMLElement, { key: 'ArrowLeft' });
    expect(onNext).not.toHaveBeenCalled();
  });

  it('leaves the arrow keys to a focused input', async () => {
    const onNext = vi.fn();
    render(
      <FilePreviewSurface
        labels={DEFAULT_FILE_PREVIEW_LABELS}
        navigation={{ index: 1, onNext, total: 5 }}
        resource={createResource({ saveText: async () => undefined })}
      />,
    );

    // 文本预览的搜索框是真实的输入控件：光标移动优先于换文件。
    const search = await screen.findByRole('searchbox');
    fireEvent.keyDown(search, { key: 'ArrowRight' });
    expect(onNext).not.toHaveBeenCalled();

    // 焦点回到面板本身时方向键才归导航。
    fireEvent.keyDown(search.closest('section') as HTMLElement, { key: 'ArrowRight' });
    expect(onNext).toHaveBeenCalledTimes(1);
  });

  it('offers no navigation affordance when the host passes none', () => {
    render(<FilePreviewSurface labels={DEFAULT_FILE_PREVIEW_LABELS} resource={createResource()} />);

    expect(screen.queryByRole('button', { name: DEFAULT_FILE_PREVIEW_LABELS.nextFile })).toBeNull();
    expect(screen.queryByRole('button', { name: DEFAULT_FILE_PREVIEW_LABELS.previousFile })).toBeNull();
  });
});
