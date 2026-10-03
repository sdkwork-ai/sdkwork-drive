import { useEffect, useId, useState } from 'react';
import type {
  SpreadsheetCellPreview,
  SpreadsheetPreviewModel,
} from '../../ooxml/spreadsheetPreview';
import {
  MAX_SPREADSHEET_PREVIEW_COLUMNS,
  MAX_SPREADSHEET_PREVIEW_ROWS,
  parseSpreadsheet,
} from '../../ooxml/spreadsheetPreview';
import type { FilePreviewLabels } from '../../i18n/filePreviewLabels';
import { formatFilePreviewLabel } from '../../i18n/filePreviewLabels';
import { describePreviewError } from '../../utils/describePreviewError';
import { PreviewErrorPanel, PreviewLoadingPanel } from '../PreviewStatePanels';
import {
  PREVIEW_BADGE_CLASS,
  PREVIEW_SCROLL_CLASS,
  PREVIEW_TOOLBAR_CLASS,
} from '../previewStyles';

export interface SpreadsheetPreviewProps {
  bytes: Uint8Array;
  /** 仅作契约保留：CSV 与 xlsx 的分支由文件名与内容探测决定，样式不随它变化。 */
  contentType?: string;
  labels: FilePreviewLabels;
  name: string;
}

type SheetModel = SpreadsheetPreviewModel['sheets'][number];

const ACTIVE_TAB_CLASS =
  'shrink-0 rounded-t-md border-b-2 border-blue-600 bg-blue-50 px-3 py-1.5 text-xs font-medium text-blue-700 dark:border-blue-400 dark:bg-blue-950/40 dark:text-blue-300';

const INACTIVE_TAB_CLASS =
  'shrink-0 rounded-t-md border-b-2 border-transparent px-3 py-1.5 text-xs font-medium text-neutral-500 transition-colors hover:bg-neutral-100 hover:text-neutral-800 dark:text-neutral-400 dark:hover:bg-neutral-800 dark:hover:text-neutral-100';

const HEADER_CELL_CLASS =
  'sticky top-0 z-10 whitespace-nowrap border-b border-neutral-200 bg-neutral-50 px-3 py-2 text-left font-medium text-neutral-700 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-200';

/**
 * 工作表是否可渲染。
 *
 * 解析器对「部件缺失或损坏」的工作表返回一个空行数组（见 `readWorksheetPreview`），
 * 而真正空白的工作表在预览里与它无从区分；给出「该工作表无法读取」比一张空白表更
 * 诚实，也符合「单张表坏掉不影响其它表」的要求。
 */
function isSheetReadable(sheet: SheetModel): boolean {
  const rows: unknown = sheet.rows;
  return Array.isArray(rows) && rows.length > 0;
}

function cellClass(cell: SpreadsheetCellPreview): string {
  const classes = ['px-3 py-1.5 align-top'];
  if (cell.isHeader) {
    classes.push('bg-neutral-50 font-medium text-neutral-800 dark:bg-neutral-900 dark:text-neutral-100');
  } else {
    classes.push('text-neutral-700 dark:text-neutral-200');
  }
  if (cell.isNumeric) {
    // 数值右对齐 + 等宽数字：小数点对齐后整列才能扫读，这是表格预览最基本的可读性。
    classes.push('text-right tabular-nums');
  }
  return classes.join(' ');
}

function SheetTable({ sheet }: { sheet: SheetModel }) {
  const rows = sheet.rows;
  const firstRow = rows.at(0);
  const hasHeaderRow = firstRow !== undefined && firstRow.some((cell) => cell.isHeader);
  const headerRow = hasHeaderRow ? firstRow : undefined;
  const bodyRows = headerRow ? rows.slice(1) : rows;

  return (
    <table className="w-full border-collapse text-left text-xs">
      {headerRow ? (
        <thead>
          <tr>
            {headerRow.map((cell, columnIndex) => (
              <th key={columnIndex} className={HEADER_CELL_CLASS} scope="col">
                {cell.text}
              </th>
            ))}
          </tr>
        </thead>
      ) : null}
      <tbody>
        {bodyRows.map((row, rowIndex) => (
          <tr
            key={rowIndex}
            // 斑马纹：列多、行多时比网格线更省视觉噪音。
            className="border-b border-neutral-100 odd:bg-white even:bg-neutral-50/70 dark:border-neutral-800 dark:odd:bg-neutral-950 dark:even:bg-neutral-900/40"
          >
            {row.map((cell, columnIndex) => (
              <td key={columnIndex} className={cellClass(cell)}>
                {cell.text}
              </td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );
}

/**
 * 表格预览：工作簿模型 + 表签 + 可横竖滚动的表格。
 *
 * `.csv` 也走这里（把文件名传给解析器），这样「逗号分隔的文本」与「xlsx 的一张表」
 * 在用户眼里是同一种东西，不必记两套交互。样式信息（单元格颜色、数字格式、合并
 * 单元格）一律不还原：读它们要带上整份 styles.xml，而且猜错的格式比不显示格式更糟。
 */
export function SpreadsheetPreview({ bytes, labels, name }: SpreadsheetPreviewProps) {
  const baseId = useId();
  const [model, setModel] = useState<SpreadsheetPreviewModel | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [pending, setPending] = useState(true);
  const [attempt, setAttempt] = useState(0);
  // 只保存「用户点了哪张表」，默认表由模型给出——默认值属于派生数据，不必再存一份状态。
  const [selectedSheet, setSelectedSheet] = useState<number | undefined>(undefined);

  useEffect(() => {
    let active = true;
    setPending(true);
    setError(undefined);

    parseSpreadsheet(bytes, name)
      .then((result) => {
        if (!active) {
          return;
        }
        setModel(result);
        setSelectedSheet(undefined);
        setPending(false);
      })
      .catch((cause: unknown) => {
        if (!active) {
          return;
        }
        setModel(undefined);
        setError(describePreviewError(cause, labels));
        setPending(false);
      });

    return () => {
      active = false;
    };
  }, [attempt, bytes, labels.loadFailed, name]);

  if (error !== undefined) {
    return (
      <PreviewErrorPanel
        labels={labels}
        message={error}
        onRetry={() => setAttempt((value) => value + 1)}
      />
    );
  }

  if (pending || !model) {
    return <PreviewLoadingPanel labels={labels} />;
  }

  const sheetCount = model.sheets.length;
  const activeIndex = Math.min(
    Math.max(selectedSheet ?? model.activeSheetIndex, 0),
    Math.max(sheetCount - 1, 0),
  );
  const activeSheet = model.sheets.at(activeIndex);

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className={PREVIEW_TOOLBAR_CLASS}>
        <span className="min-w-0 flex-1 truncate text-xs text-neutral-500 dark:text-neutral-400" title={name}>
          {name}
        </span>
        <span className={PREVIEW_BADGE_CLASS}>
          {formatFilePreviewLabel(labels.sheetCount, { count: sheetCount })}
        </span>
      </div>

      {sheetCount > 1 ? (
        <div
          aria-label={formatFilePreviewLabel(labels.sheetCount, { count: sheetCount })}
          className="flex shrink-0 gap-1 overflow-x-auto border-b border-neutral-200 bg-white px-2 pt-1 dark:border-neutral-800 dark:bg-neutral-900"
          role="tablist"
        >
          {model.sheets.map((sheet, index) => (
            <button
              key={`${sheet.name}-${index}`}
              aria-controls={`${baseId}-panel`}
              aria-selected={index === activeIndex}
              className={index === activeIndex ? ACTIVE_TAB_CLASS : INACTIVE_TAB_CLASS}
              id={`${baseId}-tab-${index}`}
              onClick={() => setSelectedSheet(index)}
              role="tab"
              title={`${labels.sheetLabel}: ${sheet.name}`}
              type="button"
            >
              <span className="block max-w-40 truncate">{sheet.name}</span>
            </button>
          ))}
        </div>
      ) : null}

      <div
        aria-labelledby={sheetCount > 1 ? `${baseId}-tab-${activeIndex}` : undefined}
        className={`${PREVIEW_SCROLL_CLASS} bg-white dark:bg-neutral-950`}
        id={`${baseId}-panel`}
        role={sheetCount > 1 ? 'tabpanel' : undefined}
      >
        {activeSheet && isSheetReadable(activeSheet) ? (
          <SheetTable sheet={activeSheet} />
        ) : (
          <p className="px-4 py-10 text-center text-xs text-neutral-500 dark:text-neutral-400">
            {labels.unsupportedSheet}
          </p>
        )}
      </div>

      {/* 行与列的截断都必须说出来：静默丢掉半张表会让人以为数据就这么点。 */}
      {activeSheet && isSheetReadable(activeSheet)
      && (activeSheet.truncatedRows || activeSheet.truncatedColumns) ? (
        <p className="shrink-0 border-t border-amber-200 bg-amber-50 px-3 py-1.5 text-center text-[11px] text-amber-700 dark:border-amber-900/50 dark:bg-amber-950/30 dark:text-amber-300">
          {[
            activeSheet.truncatedRows
              ? formatFilePreviewLabel(labels.rowsTruncated, { count: MAX_SPREADSHEET_PREVIEW_ROWS })
              : null,
            activeSheet.truncatedColumns
              ? formatFilePreviewLabel(labels.columnsTruncated, {
                  count: MAX_SPREADSHEET_PREVIEW_COLUMNS,
                })
              : null,
          ]
            .filter(Boolean)
            .join(' · ')}
        </p>
      ) : null}
    </div>
  );
}
