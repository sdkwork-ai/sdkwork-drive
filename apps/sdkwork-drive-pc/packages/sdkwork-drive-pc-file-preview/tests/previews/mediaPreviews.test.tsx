/* @vitest-environment jsdom */

import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { AudioFilePreview } from '../../src/components/previews/AudioFilePreview';
import { VideoFilePreview } from '../../src/components/previews/VideoFilePreview';
import { DEFAULT_FILE_PREVIEW_LABELS as labels } from '../../src/i18n/filePreviewLabels';

/**
 * jsdom 没有实现媒体播放：不打桩的话 `play()`/`pause()` 会把
 * "Not implemented: HTMLMediaElement.prototype.play" 打到虚拟控制台，并让
 * 「按状态先行」的断言看起来像真的播放了。打桩只替换宿主能力，不改组件逻辑。
 */
beforeEach(() => {
  vi.spyOn(HTMLMediaElement.prototype, 'play').mockResolvedValue(undefined);
  vi.spyOn(HTMLMediaElement.prototype, 'pause').mockImplementation(() => undefined);
});

afterEach(() => {
  vi.restoreAllMocks();
  cleanup();
});

describe('VideoFilePreview', () => {
  it('renders the native video element against the resolved source url', () => {
    const { container } = render(
      <VideoFilePreview labels={labels} name="clip.mp4" source={{ url: 'blob:video-1' }} />,
    );

    const video = container.querySelector('video');
    expect(video).not.toBeNull();
    expect(video?.getAttribute('src')).toBe('blob:video-1');
    expect(video?.getAttribute('preload')).toBe('metadata');
    expect(video?.hasAttribute('controls')).toBe(true);
    expect(video?.className).toContain('max-h-full');
    // 时间读数用 `mm:ss / mm:ss` 呈现，未播放时两端都是 00:00。
    expect(screen.getByText('00:00 / 00:00')).toBeDefined();
  });

  it('toggles aria-pressed on the play/pause button and respects the speed select', () => {
    render(<VideoFilePreview labels={labels} name="clip.mp4" source={{ url: 'blob:video-1' }} />);

    const playButton = screen.getByRole('button', { name: labels.play });
    expect(playButton.getAttribute('aria-pressed')).toBe('false');

    fireEvent.click(playButton);
    const pauseButton = screen.getByRole('button', { name: labels.pause });
    expect(pauseButton.getAttribute('aria-pressed')).toBe('true');

    fireEvent.change(screen.getByRole('combobox', { name: labels.playbackRate }), {
      target: { value: '1.5' },
    });
    expect(screen.getByRole('combobox', { name: labels.playbackRate })).toHaveProperty('value', '1.5');

    fireEvent.click(pauseButton);
    expect(screen.getByRole('button', { name: labels.play }).getAttribute('aria-pressed')).toBe('false');
  });
});

describe('AudioFilePreview', () => {
  it('renders the hidden audio element and its custom controls', () => {
    const { container } = render(
      <AudioFilePreview labels={labels} name="song.mp3" source={{ url: 'blob:audio-1' }} />,
    );

    const audio = container.querySelector('audio');
    expect(audio).not.toBeNull();
    expect(audio?.getAttribute('src')).toBe('blob:audio-1');
    expect(audio?.getAttribute('preload')).toBe('metadata');
    expect(screen.getByRole('slider', { name: labels.volume })).toBeDefined();
    expect(screen.getByRole('combobox', { name: labels.playbackRate })).toBeDefined();
  });

  it('toggles aria-pressed from the button and from the space key on the player', () => {
    render(<AudioFilePreview labels={labels} name="song.mp3" source={{ url: 'blob:audio-1' }} />);

    const playButton = screen.getByRole('button', { name: labels.play });
    fireEvent.click(playButton);
    expect(screen.getByRole('button', { name: labels.pause }).getAttribute('aria-pressed')).toBe('true');

    // 只有播放器自己持有焦点时空格才生效，不劫持宿主页面的全局键盘。
    fireEvent.keyDown(screen.getByRole('group'), { key: ' ' });
    expect(screen.getByRole('button', { name: labels.play }).getAttribute('aria-pressed')).toBe('false');
  });

  it('consumes the arrow keys so file navigation never double-fires', () => {
    // 播放器与预览面板都关心方向键：播放器用于快进快退，面板用于切换相邻文件。
    // 播放器消费掉的按键必须停止冒泡，否则一次按键会同时做两件事。
    const onAncestorKeyDown = vi.fn();
    render(
      // eslint-disable-next-line jsx-a11y/no-static-element-interactions
      <div onKeyDown={onAncestorKeyDown}>
        <VideoFilePreview labels={labels} name="clip.mp4" source={{ url: 'blob:video-1' }} />
      </div>,
    );

    fireEvent.keyDown(screen.getByRole('group'), { key: 'ArrowRight' });
    fireEvent.keyDown(screen.getByRole('group'), { key: ' ' });

    expect(onAncestorKeyDown).not.toHaveBeenCalled();
  });
});
