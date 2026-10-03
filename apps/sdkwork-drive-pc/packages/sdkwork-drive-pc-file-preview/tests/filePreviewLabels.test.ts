import { describe, expect, it } from 'vitest';
import {
  DEFAULT_FILE_PREVIEW_LABELS,
  formatFilePreviewLabel,
  mergeFilePreviewLabels,
} from '../src/i18n/filePreviewLabels';

describe('mergeFilePreviewLabels', () => {
  it('returns the English defaults when a host provides nothing', () => {
    expect(mergeFilePreviewLabels(undefined)).toBe(DEFAULT_FILE_PREVIEW_LABELS);
  });

  it('overrides only the fields the host translated', () => {
    const labels = mergeFilePreviewLabels({ save: '保存' });
    expect(labels.save).toBe('保存');
    // 未翻译的字段继续走英文兜底，而不是露出 undefined。
    expect(labels.cancel).toBe(DEFAULT_FILE_PREVIEW_LABELS.cancel);
  });

  it('merges the kind table field by field', () => {
    const labels = mergeFilePreviewLabels({ kind: { pdf: 'PDF 文档' } as never });
    expect(labels.kind.pdf).toBe('PDF 文档');
    expect(labels.kind.image).toBe(DEFAULT_FILE_PREVIEW_LABELS.kind.image);
  });
});

describe('formatFilePreviewLabel', () => {
  it('replaces named placeholders', () => {
    expect(formatFilePreviewLabel('{current} / {total}', { current: 2, total: 7 })).toBe('2 / 7');
    expect(formatFilePreviewLabel('共 {count} 条目', { count: 12 })).toBe('共 12 条目');
  });

  it('leaves unknown placeholders untouched instead of printing undefined', () => {
    expect(formatFilePreviewLabel('{a}-{b}', { a: 'x' })).toBe('x-{b}');
  });

  it('returns the template unchanged without params', () => {
    expect(formatFilePreviewLabel('plain text')).toBe('plain text');
  });
});
