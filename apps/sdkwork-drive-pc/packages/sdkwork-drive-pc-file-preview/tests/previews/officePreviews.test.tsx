/* @vitest-environment jsdom */

import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { ArchivePreviewModel } from '../../src/ooxml/archivePreview';
import { readArchivePreview } from '../../src/ooxml/archivePreview';
import type { PresentationPreviewModel } from '../../src/ooxml/presentationPreview';
import { parsePresentation } from '../../src/ooxml/presentationPreview';
import type {
  SpreadsheetCellPreview,
  SpreadsheetPreviewModel,
} from '../../src/ooxml/spreadsheetPreview';
import { MAX_SPREADSHEET_PREVIEW_ROWS, parseSpreadsheet } from '../../src/ooxml/spreadsheetPreview';
import type { WordDocumentPreviewModel } from '../../src/ooxml/wordDocumentPreview';
import { parseWordDocument } from '../../src/ooxml/wordDocumentPreview';
import { ArchivePreview } from '../../src/components/previews/ArchivePreview';
import { SlidesPreview } from '../../src/components/previews/SlidesPreview';
import { SpreadsheetPreview } from '../../src/components/previews/SpreadsheetPreview';
import { WordDocumentPreview } from '../../src/components/previews/WordDocumentPreview';
import {
  DEFAULT_FILE_PREVIEW_LABELS as labels,
  formatFilePreviewLabel,
} from '../../src/i18n/filePreviewLabels';

// OOXML 解析器由并行任务实现，测试只依赖冻结的签名：全部打桩，避免测试与解析器
// 的解析细节、fixture 二进制互相绑定。
vi.mock('../../src/ooxml/wordDocumentPreview', () => ({ parseWordDocument: vi.fn() }));
vi.mock('../../src/ooxml/presentationPreview', () => ({ parsePresentation: vi.fn() }));
vi.mock('../../src/ooxml/archivePreview', () => ({ readArchivePreview: vi.fn() }));
vi.mock('../../src/ooxml/spreadsheetPreview', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/ooxml/spreadsheetPreview')>();
  // 行数上限是渲染文案的一部分（截断提示里的 count），保留真实常量才能验证文案。
  return { ...actual, parseSpreadsheet: vi.fn() };
});

const BYTES = new Uint8Array([1, 2, 3, 4]);

afterEach(() => {
  vi.restoreAllMocks();
  cleanup();
});

function cell(text: string, flags: Partial<SpreadsheetCellPreview> = {}): SpreadsheetCellPreview {
  return { text, isNumeric: false, isHeader: false, ...flags };
}

describe('WordDocumentPreview', () => {
  it('renders the document title, heading hierarchy, run styles and list indent', async () => {
    const model: WordDocumentPreviewModel = {
      title: 'Quarterly report',
      paragraphs: [
        { kind: 'heading', level: 1, runs: [{ text: 'Overview' }] },
        {
          kind: 'paragraph',
          alignment: 'center',
          runs: [{ text: 'Plain ' }, { text: 'bold', bold: true }, { text: 'code', code: true }],
        },
        { kind: 'listItem', level: 1, runs: [{ text: 'First point' }] },
      ],
    };
    vi.mocked(parseWordDocument).mockResolvedValue(model);

    const { container } = render(
      <WordDocumentPreview bytes={BYTES} labels={labels} name="report.docx" />,
    );

    expect(await screen.findByRole('heading', { level: 1, name: 'Quarterly report' })).toBeDefined();
    expect(screen.getByRole('heading', { level: 2, name: 'Overview' })).toBeDefined();
    expect(screen.getByText('bold').tagName).toBe('STRONG');
    expect(screen.getByText('code').tagName).toBe('CODE');
    expect(screen.getByText('Plain').className).toContain('text-center');
    // 一级列表缩进 = 1 × 16px。
    expect(container.querySelector('[style*="padding-left: 16px"]')).not.toBeNull();
  });

  it('shows the error panel when the parser rejects', async () => {
    vi.mocked(parseWordDocument).mockRejectedValue(new Error('word/document.xml 损坏'));

    render(<WordDocumentPreview bytes={BYTES} labels={labels} name="broken.docx" />);

    expect(await screen.findByText(labels.loadFailed)).toBeDefined();
    expect(screen.getByText('word/document.xml 损坏')).toBeDefined();
    expect(screen.getByRole('button', { name: labels.retry })).toBeDefined();
  });
});

describe('SpreadsheetPreview', () => {
  const model: SpreadsheetPreviewModel = {
    activeSheetIndex: 0,
    sheets: [
      {
        name: 'Summary',
        truncatedRows: true,
        truncatedColumns: false,
        rows: [
          [cell('Item', { isHeader: true }), cell('Amount', { isHeader: true })],
          [cell('Hosting'), cell('1200', { isNumeric: true })],
        ],
      },
      { name: 'Damaged', truncatedRows: false, truncatedColumns: false, rows: [] },
    ],
  };

  it('renders sheet tabs, right-aligned numbers and the truncation notice', async () => {
    vi.mocked(parseSpreadsheet).mockResolvedValue(model);

    render(
      <SpreadsheetPreview bytes={BYTES} contentType="text/csv" labels={labels} name="budget.csv" />,
    );

    const summaryTab = await screen.findByRole('tab', { name: 'Summary' });
    expect(screen.getByRole('tab', { name: 'Damaged' })).toBeDefined();
    expect(summaryTab.getAttribute('aria-selected')).toBe('true');
    expect(screen.getByRole('columnheader', { name: 'Amount' }).className).toContain('font-medium');
    expect(screen.getByText('1200').className).toContain('text-right');
    expect(screen.getByText('1200').className).toContain('tabular-nums');
    expect(
      screen.getByText(
        formatFilePreviewLabel(labels.rowsTruncated, { count: MAX_SPREADSHEET_PREVIEW_ROWS }),
      ),
    ).toBeDefined();
    // CSV 没有工作表名，文件名必须传给解析器才能得到正确的表签。
    expect(vi.mocked(parseSpreadsheet)).toHaveBeenCalledWith(BYTES, 'budget.csv');
  });

  it('keeps the other sheets when a single sheet cannot be read', async () => {
    vi.mocked(parseSpreadsheet).mockResolvedValue(model);

    render(<SpreadsheetPreview bytes={BYTES} labels={labels} name="budget.xlsx" />);

    fireEvent.click(await screen.findByRole('tab', { name: 'Damaged' }));

    expect(screen.getByText(labels.unsupportedSheet)).toBeDefined();
    expect(screen.queryByRole('columnheader', { name: 'Amount' })).toBeNull();
    // 坏掉一张表不会丢掉整份预览：其它表签仍在。
    expect(screen.getByRole('tab', { name: 'Summary' })).toBeDefined();
  });
});

describe('SlidesPreview', () => {
  const model: PresentationPreviewModel = {
    slides: [
      { index: 1, title: 'Intro', lines: ['Welcome', 'Agenda'] },
      { index: 2, title: 'Roadmap', lines: ['Q1 launch'] },
    ],
  };

  it('renders the rail, switches slides by click and by ArrowRight', async () => {
    vi.mocked(parsePresentation).mockResolvedValue(model);

    render(<SlidesPreview bytes={BYTES} labels={labels} name="deck.pptx" />);

    const firstSlide = await screen.findByRole('button', {
      name: `${labels.slideLabel} 1: Intro`,
    });
    const secondSlide = screen.getByRole('button', { name: `${labels.slideLabel} 2: Roadmap` });
    expect(firstSlide.getAttribute('aria-current')).toBe('true');
    expect(screen.getByText(formatFilePreviewLabel(labels.slideCount, { count: 2 }))).toBeDefined();
    expect(screen.getByRole('heading', { name: 'Intro' })).toBeDefined();

    fireEvent.click(secondSlide);
    expect(screen.getByRole('heading', { name: 'Roadmap' })).toBeDefined();
    expect(secondSlide.getAttribute('aria-current')).toBe('true');
    // 末页时「下一页」置灰。
    expect(screen.getByRole('button', { name: labels.next }).hasAttribute('disabled')).toBe(true);

    const player = screen.getByRole('group');
    fireEvent.keyDown(player, { key: 'ArrowLeft' });
    expect(screen.getByRole('heading', { name: 'Intro' })).toBeDefined();
    fireEvent.keyDown(player, { key: 'ArrowRight' });
    expect(screen.getByRole('heading', { name: 'Roadmap' })).toBeDefined();
  });
});

describe('ArchivePreview', () => {
  const model: ArchivePreviewModel = {
    entries: [
      { path: 'image.png', isDirectory: false, uncompressedSize: 4096, compressedSize: 4096 },
      { path: 'docs/', isDirectory: true, uncompressedSize: 0, compressedSize: 0 },
      { path: 'docs/report.pdf', isDirectory: false, uncompressedSize: 2048, compressedSize: 1024 },
    ],
    totalUncompressedBytes: 6144,
    truncated: true,
  };

  it('lists directories first and reports the truncated listing', async () => {
    vi.mocked(readArchivePreview).mockReturnValue(model);

    const { container } = render(
      <ArchivePreview bytes={BYTES} labels={labels} name="bundle.zip" />,
    );

    await waitFor(() => expect(container.querySelectorAll('tbody tr')).toHaveLength(3));

    const rows = Array.from(container.querySelectorAll('tbody tr'));
    // 目录优先，其余按路径排序：写入顺序是打包工具的产物，不能照抄给用户。
    expect(rows[0]?.textContent).toContain('docs/');
    expect(rows[1]?.textContent).toContain('docs/report.pdf');
    expect(rows[2]?.textContent).toContain('image.png');
    expect(rows[0]?.querySelector('[data-file-kind]')?.getAttribute('data-file-kind')).toBe('folder');
    expect(rows[2]?.querySelector('[data-file-kind]')?.getAttribute('data-file-kind')).toBe('image');
    // 目录行不显示尺寸，只占位。
    expect(rows[0]?.textContent).toContain('—');
    expect(
      screen.getByText(formatFilePreviewLabel(labels.archiveEntries, { count: 3 })),
    ).toBeDefined();
    expect(
      screen.getByText(formatFilePreviewLabel(labels.archiveTruncated, { count: 3 })),
    ).toBeDefined();
  });

  it('falls back to the error panel for an unreadable archive', () => {
    vi.mocked(readArchivePreview).mockImplementation(() => {
      throw new Error('central directory is broken');
    });

    render(<ArchivePreview bytes={BYTES} labels={labels} name="broken.zip" />);

    expect(screen.getByText(labels.loadFailed)).toBeDefined();
    expect(screen.getByText('central directory is broken')).toBeDefined();
  });
});
