/* @vitest-environment jsdom */

import React from 'react';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { TextFilePreview } from '../src/components/previews/TextFilePreview';
import { DEFAULT_FILE_PREVIEW_LABELS } from '../src/i18n/filePreviewLabels';

afterEach(() => cleanup());

const LINES = 'first line\nsecond line\nthird line';

describe('TextFilePreview', () => {
  it('renders the text with line numbers and statistics', () => {
    render(
      <TextFilePreview labels={DEFAULT_FILE_PREVIEW_LABELS} name="notes.txt" text={LINES} />,
    );

    expect(screen.getByText('first line')).toBeTruthy();
    expect(screen.getByText('third line')).toBeTruthy();
    // 行号栏是独立的一列，三行文本就有三个行号。
    expect(screen.getByText('3')).toBeTruthy();
    expect(screen.getByText(`${DEFAULT_FILE_PREVIEW_LABELS.linesLabel}: 3`)).toBeTruthy();
  });

  it('searches inside the file and reports the match position', () => {
    render(
      <TextFilePreview labels={DEFAULT_FILE_PREVIEW_LABELS} name="notes.txt" text={LINES} />,
    );

    fireEvent.change(screen.getByRole('searchbox'), { target: { value: 'line' } });

    // 三行都命中，计数按 `{current} / {total}` 显示。
    expect(screen.getByText('1 / 3')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: DEFAULT_FILE_PREVIEW_LABELS.next }));
    expect(screen.getByText('2 / 3')).toBeTruthy();
  });

  it('reports no match instead of an empty counter', () => {
    render(
      <TextFilePreview labels={DEFAULT_FILE_PREVIEW_LABELS} name="notes.txt" text={LINES} />,
    );

    fireEvent.change(screen.getByRole('searchbox'), { target: { value: 'nothing-here' } });
    expect(screen.getByText(DEFAULT_FILE_PREVIEW_LABELS.noMatches)).toBeTruthy();
  });

  it('toggles line wrapping from the toolbar', () => {
    render(
      <TextFilePreview labels={DEFAULT_FILE_PREVIEW_LABELS} name="notes.txt" text={LINES} />,
    );

    const wrapButton = screen.getByRole('button', { name: DEFAULT_FILE_PREVIEW_LABELS.wrapLines });
    expect(wrapButton.getAttribute('aria-pressed')).toBe('false');
    fireEvent.click(wrapButton);
    expect(
      screen
        .getByRole('button', { name: DEFAULT_FILE_PREVIEW_LABELS.unwrapLines })
        .getAttribute('aria-pressed'),
    ).toBe('true');
  });

  it('handles an empty document without crashing', () => {
    render(<TextFilePreview labels={DEFAULT_FILE_PREVIEW_LABELS} name="empty.txt" text="" />);
    expect(screen.getByText(`${DEFAULT_FILE_PREVIEW_LABELS.charactersLabel}: 0`)).toBeTruthy();
  });
});
