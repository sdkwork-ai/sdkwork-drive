/**
 * `.xlsx` / `.csv` 到表格预览视图模型的转换。
 *
 * 这里的取舍是「排版信息一律不解析」：单元格样式、日期格式、合并单元格都在
 * `styles.xml` 与工作表的 `mergeCells` 里，读它们需要一整套样式索引与格式码解析，
 * 而预览只需要显示「哪一格写了什么」。因此日期会以 xlsx 内部序列号呈现——渲染层
 * 若需要，应当由知道列语义的调用方补格式，而不是让预览层猜。
 */

import { ZipArchiveError, isZipArchive, readZipArchive, type ZipArchive } from './zipArchive';

/** SpreadsheetML 主命名空间（xlsx 的工作表用默认命名空间，元素不带前缀）。 */
const SPREADSHEET_NAMESPACE = 'http://schemas.openxmlformats.org/spreadsheetml/2006/main';

/** Office 文档关系命名空间，`r:id` 在这里。 */
const OFFICE_RELATIONSHIPS_NAMESPACE = 'http://schemas.openxmlformats.org/officeDocument/2006/relationships';

/** 包级关系命名空间，`xl/_rels/workbook.xml.rels` 的根元素用它。 */
const PACKAGE_RELATIONSHIPS_NAMESPACE = 'http://schemas.openxmlformats.org/package/2006/relationships';

/** 工作簿部件与它的关系部件路径。 */
const WORKBOOK_PART_PATH = 'xl/workbook.xml';
const WORKBOOK_RELATIONSHIPS_PART_PATH = 'xl/_rels/workbook.xml.rels';
const SHARED_STRINGS_PART_PATH = 'xl/sharedStrings.xml';

/** 工作表关系部件所在目录，用于把关系里的相对路径补全成归档路径。 */
const WORKSHEET_BASE_DIRECTORY = 'xl';

/** 预览行上限：够看清表头与若干行数据，又不至于把整张表都渲染进 DOM。 */
export const MAX_SPREADSHEET_PREVIEW_ROWS = 500;

/** 预览列上限：再宽的表格在预览面板里也只能横向滚动，不如截断。 */
export const MAX_SPREADSHEET_PREVIEW_COLUMNS = 64;

/** 单元格引用里的列字母，例如 `C3` 的 `C`。 */
const COLUMN_REFERENCE_PATTERN = /^([A-Za-z]+)/;

/**
 * CSV 数值判定。
 *
 * 用严格的全串匹配而不是 `Number.parseFloat`：后者会把 `12abc`、`1,234`、`007` 里
 * 的前缀当数字，预览里就会出现「文本被当成数值右对齐」的错位。
 */
const NUMERIC_TEXT_PATTERN = /^[+-]?(?:\d+(?:\.\d+)?|\.\d+)(?:[eE][+-]?\d+)?$/;

export interface SpreadsheetCellPreview {
  text: string;
  isNumeric: boolean;
  /** 第 1 行视为表头（渲染用）。 */
  isHeader: boolean;
}

export interface SpreadsheetSheetPreview {
  name: string;
  rows: SpreadsheetCellPreview[][];
  /** 超过行上限时为 true。 */
  truncatedRows: boolean;
  /** 超过列上限时为 true；与行截断一样必须让用户知道还有内容没显示。 */
  truncatedColumns: boolean;
}

export interface SpreadsheetPreviewModel {
  sheets: SpreadsheetSheetPreview[];
  activeSheetIndex: number;
}

export type SpreadsheetPreviewErrorCode =
  | 'dom-parser-unavailable'
  | 'workbook-xml-invalid'
  | 'workbook-missing';

/** 工作簿级解析失败；单个工作表损坏只丢那一张表，不会用这个错误。 */
export class SpreadsheetPreviewError extends Error {
  readonly code: SpreadsheetPreviewErrorCode;

  constructor(code: SpreadsheetPreviewErrorCode, message: string) {
    super(message);
    this.name = 'SpreadsheetPreviewError';
    this.code = code;
  }
}

/**
 * 解析 `.xlsx` 或 `.csv`。
 *
 * `fileName` 既用于判断扩展名，也用于给 CSV 的伪工作表命名：CSV 没有工作表名，
 * 用文件名当标签比凭空造一个 `Sheet1` 更利于用户确认自己打开的是哪个文件。
 */
export async function parseSpreadsheet(
  bytes: Uint8Array,
  fileName?: string,
): Promise<SpreadsheetPreviewModel> {
  const csvSheetName = resolveCsvSheetName(fileName);
  if (isDelimitedFileName(fileName) || !isZipArchive(bytes)) {
    return parseCsvSpreadsheet(decodeTextBytes(bytes), csvSheetName, resolveDelimitedSeparator(fileName, bytes));
  }

  const archive = readZipArchive(bytes);
  const workbookXml = await archive.readText(WORKBOOK_PART_PATH);
  if (workbookXml === undefined) {
    // 是 ZIP 却没有工作簿部件：`.ods`、Numbers、损坏的 xlsx 都长这样。
    // 以前这里退回按文本解析，等于把压缩容器的字节解码成一张乱码表——宁可明确报错。
    throw new SpreadsheetPreviewError(
      'workbook-missing',
      `${fileLabel(fileName)} is a ZIP container without ${WORKBOOK_PART_PATH}`,
    );
  }
  return parseWorkbookArchive(archive, workbookXml);
}

/** 解析 RFC 4180 风格的 CSV/TSV 文本；`sheetName` 缺省为 `Sheet1`。 */
export function parseCsvSpreadsheet(
  text: string,
  sheetName?: string,
  separator: string = ',',
): SpreadsheetPreviewModel {
  const parsed = readCsvRecords(text, separator);
  const rows = parsed.rows
    .slice(0, MAX_SPREADSHEET_PREVIEW_ROWS)
    .map((values, rowIndex) => values
      .slice(0, MAX_SPREADSHEET_PREVIEW_COLUMNS)
      .map((value) => createCellPreview(value, isNumericText(value), rowIndex === 0)));

  return {
    sheets: [{
      name: sheetName ?? 'Sheet1',
      rows,
      truncatedRows: parsed.truncated,
      truncatedColumns: parsed.rows.some((values) => values.length > MAX_SPREADSHEET_PREVIEW_COLUMNS),
    }],
    activeSheetIndex: 0,
  };
}

async function parseWorkbookArchive(
  archive: ZipArchive,
  workbookXml: string,
): Promise<SpreadsheetPreviewModel> {
  const document = parseSpreadsheetXml(workbookXml, WORKBOOK_PART_PATH);
  const relationships = await readWorkbookRelationships(archive);
  const sharedStrings = await readSharedStrings(archive);

  const sheets: SpreadsheetSheetPreview[] = [];
  const sheetElements = collectElements(document, 'sheet');
  for (const [index, sheetElement] of sheetElements.entries()) {
    const name = sheetElement.getAttribute('name') ?? `Sheet${index + 1}`;
    const partPath = resolveWorksheetPartPath(sheetElement, relationships, index);
    sheets.push(readWorksheetPreview(name, await archive.readText(partPath), sharedStrings));
  }

  return { sheets, activeSheetIndex: readActiveSheetIndex(document, sheets.length) };
}

function readWorksheetPreview(
  name: string,
  xml: string | undefined,
  sharedStrings: readonly string[],
): SpreadsheetSheetPreview {
  const document = xml === undefined ? undefined : tryParseSpreadsheetXml(xml);
  const sheetData = document ? findFirstElement(document, 'sheetData') : undefined;
  if (!sheetData) {
    // 工作表部件缺失或损坏时保留一个空表：其它工作表仍然可以正常预览。
    return { name, rows: [], truncatedRows: false, truncatedColumns: false };
  }

  const rowElements = childElements(sheetData).filter((child) => isSpreadsheetElement(child, 'row'));
  const truncatedRows = rowElements.length > MAX_SPREADSHEET_PREVIEW_ROWS;
  const rows: SpreadsheetCellPreview[][] = [];
  let truncatedColumns = false;
  for (const rowElement of rowElements.slice(0, MAX_SPREADSHEET_PREVIEW_ROWS)) {
    // 列超限要在行内判定：`readRowElement` 会丢掉超出上限的单元格，丢了多少只有它知道。
    if (rowElement.getElementsByTagName('c').length > MAX_SPREADSHEET_PREVIEW_COLUMNS) {
      truncatedColumns = true;
    }
    rows.push(readRowElement(rowElement, sharedStrings, rows.length === 0));
  }
  return { name, rows, truncatedRows, truncatedColumns };
}

/**
 * 读取一行单元格，并按 `r` 里的列字母把稀疏单元格放回真实列位。
 *
 * Excel 不会为空白单元格写 `<c>`，所以 `C3` 前面可能没有 `B3`；只按下标顺序追加
 * 会让 C 列的内容左移到 B 列，整行错位。
 */
function readRowElement(
  rowElement: Element,
  sharedStrings: readonly string[],
  isHeader: boolean,
): SpreadsheetCellPreview[] {
  const cells: SpreadsheetCellPreview[] = [];
  let nextColumnIndex = 0;
  for (const child of childElements(rowElement)) {
    if (!isSpreadsheetElement(child, 'c')) {
      continue;
    }
    const columnIndex = readColumnIndex(child.getAttribute('r'), nextColumnIndex);
    nextColumnIndex = columnIndex + 1;
    if (columnIndex < 0 || columnIndex >= MAX_SPREADSHEET_PREVIEW_COLUMNS) {
      continue;
    }
    while (cells.length < columnIndex) {
      cells.push(createCellPreview('', false, isHeader));
    }
    cells[columnIndex] = readCellElement(child, sharedStrings, isHeader);
  }
  return cells;
}

function readCellElement(
  element: Element,
  sharedStrings: readonly string[],
  isHeader: boolean,
): SpreadsheetCellPreview {
  const type = element.getAttribute('t');

  if (type === 's') {
    const index = Number.parseInt(readChildElementText(element, 'v') ?? '', 10);
    const text = Number.isInteger(index) && index >= 0 ? sharedStrings[index] ?? '' : '';
    return createCellPreview(text, false, isHeader);
  }

  if (type === 'inlineStr') {
    const inline = findChildElement(element, 'is');
    const parts: string[] = [];
    if (inline) {
      collectTextParts(inline, parts);
    }
    return createCellPreview(parts.join(''), false, isHeader);
  }

  const raw = readChildElementText(element, 'v') ?? '';
  // 未标注类型或 `n` 表示数值（日期在 xlsx 里同样是数值，格式信息在样式中）。
  // `str`/`b`/`e` 都是文本语义，原样展示即可。
  const isNumeric = (type === null || type === 'n') && raw.length > 0;
  return createCellPreview(raw, isNumeric, isHeader);
}

function createCellPreview(text: string, isNumeric: boolean, isHeader: boolean): SpreadsheetCellPreview {
  return { text, isNumeric, isHeader };
}

function readColumnIndex(reference: string | null, fallback: number): number {
  const match = reference === null ? null : COLUMN_REFERENCE_PATTERN.exec(reference);
  const letters = (match?.[1] ?? '').toUpperCase();
  if (letters.length === 0) {
    return fallback;
  }
  let index = 0;
  for (const letter of letters) {
    index = index * 26 + (letter.charCodeAt(0) - 64);
  }
  return index - 1;
}

async function readSharedStrings(archive: ZipArchive): Promise<string[]> {
  const xml = await archive.readText(SHARED_STRINGS_PART_PATH);
  if (xml === undefined) {
    return [];
  }
  const document = tryParseSpreadsheetXml(xml);
  if (!document) {
    return [];
  }
  const strings: string[] = [];
  for (const child of childElements(document.documentElement)) {
    if (isSpreadsheetElement(child, 'si')) {
      const parts: string[] = [];
      collectTextParts(child, parts);
      strings.push(parts.join(''));
    }
  }
  return strings;
}

/**
 * 收集 `si` / `is` 里的文本。
 *
 * 富文本把内容拆进多个 `<r><t>`，拼接才是单元格的完整值；`<rPh>` 是给东亚文字标注的
 * 拼音/注音提示，不属于单元格内容，必须整棵跳过。
 */
function collectTextParts(root: Element, parts: string[]): void {
  for (const child of childElements(root)) {
    if (isSpreadsheetElement(child, 'rPh')) {
      continue;
    }
    if (isSpreadsheetElement(child, 't')) {
      parts.push(child.textContent ?? '');
      continue;
    }
    collectTextParts(child, parts);
  }
}

async function readWorkbookRelationships(archive: ZipArchive): Promise<ReadonlyMap<string, string>> {
  const targets = new Map<string, string>();
  const xml = await archive.readText(WORKBOOK_RELATIONSHIPS_PART_PATH);
  if (xml === undefined) {
    return targets;
  }

  const document = tryParseSpreadsheetXml(xml);
  if (!document) {
    return targets;
  }
  for (const relationship of collectElements(document, 'Relationship')) {
    if (relationship.getAttribute('TargetMode') === 'External') {
      continue;
    }
    const id = relationship.getAttribute('Id');
    const target = relationship.getAttribute('Target');
    if (id && target) {
      targets.set(id, target);
    }
  }
  return targets;
}

function resolveWorksheetPartPath(
  sheetElement: Element,
  relationships: ReadonlyMap<string, string>,
  index: number,
): string {
  const relationshipId = sheetElement.getAttributeNS(OFFICE_RELATIONSHIPS_NAMESPACE, 'id')
    ?? sheetElement.getAttribute('r:id');
  const target = relationshipId ? relationships.get(relationshipId) : undefined;
  if (target === undefined) {
    // 关系缺失时退回约定路径：不该因为一个 rels 部件的问题让整份表格空白。
    return `xl/worksheets/sheet${index + 1}.xml`;
  }
  return resolveRelationshipTarget(target, WORKSHEET_BASE_DIRECTORY);
}

/** 关系目标可能是 `worksheets/sheet1.xml`（相对 `xl/`）或 `/xl/worksheets/sheet1.xml`（绝对）。 */
function resolveRelationshipTarget(target: string, baseDirectory: string): string {
  const absolute = target.startsWith('/') ? target.slice(1) : `${baseDirectory}/${target}`;
  const segments: string[] = [];
  for (const segment of absolute.split('/')) {
    if (segment === '' || segment === '.') {
      continue;
    }
    if (segment === '..') {
      segments.pop();
      continue;
    }
    segments.push(segment);
  }
  return segments.join('/');
}

function readActiveSheetIndex(document: XMLDocument, sheetCount: number): number {
  const activeTab = findFirstElement(document, 'workbookView')?.getAttribute('activeTab');
  const parsed = activeTab === null || activeTab === undefined ? Number.NaN : Number.parseInt(activeTab, 10);
  if (!Number.isInteger(parsed) || parsed < 0 || parsed >= sheetCount) {
    return 0;
  }
  return parsed;
}

interface CsvRecords {
  rows: string[][];
  truncated: boolean;
}

/**
 * 解析 RFC 4180 风格的分隔符文本（CSV/TSV）。
 *
 * 手写状态机而不是按行 `split`：引号字段里允许出现换行，按行切会把一条记录拆成两条。
 * 多解析一行用来判断「是否还有更多数据」，从而给出准确的 `truncatedRows`。
 */
function readCsvRecords(text: string, separator: string): CsvRecords {
  const rows: string[][] = [];
  let row: string[] = [];
  let field = '';
  let insideQuotes = false;
  let index = 0;

  while (index < text.length) {
    const character = text.charAt(index);

    if (insideQuotes) {
      if (character === '"') {
        if (text.charAt(index + 1) === '"') {
          field += '"';
          index += 2;
          continue;
        }
        insideQuotes = false;
        index += 1;
        continue;
      }
      field += character;
      index += 1;
      continue;
    }

    if (character === '"' && field.length === 0) {
      insideQuotes = true;
      index += 1;
      continue;
    }
    if (character === separator) {
      row.push(field);
      field = '';
      index += 1;
      continue;
    }
    if (character === '\r' || character === '\n') {
      if (character === '\r' && text.charAt(index + 1) === '\n') {
        index += 1;
      }
      row.push(field);
      rows.push(row);
      row = [];
      field = '';
      index += 1;
      if (rows.length > MAX_SPREADSHEET_PREVIEW_ROWS) {
        return { rows, truncated: true };
      }
      continue;
    }

    field += character;
    index += 1;
  }

  // 收尾：只有确实还有未提交的字段/列时才补最后一行，避免把结尾换行当成空行。
  if (field.length > 0 || row.length > 0) {
    row.push(field);
    rows.push(row);
  }
  return { rows, truncated: false };
}

function isNumericText(value: string): boolean {
  return NUMERIC_TEXT_PATTERN.test(value);
}

/** `.csv` 与 `.tsv` 都是分隔符文本，只是分隔符不同。 */
function isDelimitedFileName(fileName: string | undefined): boolean {
  if (fileName === undefined) {
    return false;
  }
  const lowered = fileName.toLowerCase();
  return lowered.endsWith('.csv') || lowered.endsWith('.tsv');
}

/**
 * 解析分隔符：扩展名优先，否则嗅探第一条记录。
 *
 * `.tsv` 里用逗号分隔是错的（值里本来就可能含逗号），而 Excel 在部分区域设置下导出的
 * CSV 用分号。嗅探只在样本里数一次出现次数，取最多的那个，比按扩展名硬编码更耐脏数据。
 */
function resolveDelimitedSeparator(fileName: string | undefined, bytes: Uint8Array): string {
  if (fileName !== undefined && fileName.toLowerCase().endsWith('.tsv')) {
    return '\t';
  }
  const sample = decodeTextBytes(bytes).slice(0, 4096).split(/\r?\n/, 1)[0] ?? '';
  const candidates = ['\t', ';', ','];
  let best = ',';
  let bestCount = 0;
  for (const candidate of candidates) {
    const count = sample.split(candidate).length - 1;
    if (count > bestCount) {
      best = candidate;
      bestCount = count;
    }
  }
  return best;
}

/** 错误信息里引用文件名的可读形式。 */
function fileLabel(fileName: string | undefined): string {
  return fileName === undefined ? 'spreadsheet' : fileName.split(/[\\/]/).pop() ?? fileName;
}

/** 分隔符文本没有工作表名，用去掉扩展名的文件名当标签。 */
function resolveCsvSheetName(fileName: string | undefined): string {
  if (fileName === undefined) {
    return 'Sheet1';
  }
  const baseName = fileName.split(/[\\/]/).pop() ?? '';
  const withoutExtension = baseName.replace(/\.[^.]*$/, '');
  return withoutExtension.length > 0 ? withoutExtension : 'Sheet1';
}

function decodeTextBytes(bytes: Uint8Array): string {
  return new TextDecoder('utf-8').decode(bytes).replace(/^\uFEFF/, '');
}

function parseSpreadsheetXml(xml: string, partPath: string): XMLDocument {
  const document = tryParseSpreadsheetXml(xml);
  if (!document) {
    throw new SpreadsheetPreviewError('workbook-xml-invalid', `${partPath} 不是可解析的 XML`);
  }
  return document;
}

function tryParseSpreadsheetXml(xml: string): XMLDocument | undefined {
  if (typeof DOMParser !== 'function') {
    throw new SpreadsheetPreviewError(
      'dom-parser-unavailable',
      '解析 Excel 工作簿需要 DOMParser：请在浏览器或 jsdom 等带 DOM 的运行时中调用 parseSpreadsheet',
    );
  }
  const document = new DOMParser().parseFromString(xml, 'application/xml');
  return isParserErrorDocument(document) ? undefined : document;
}

function isParserErrorDocument(document: XMLDocument): boolean {
  const root = document.documentElement;
  return root !== null && root.localName === 'parsererror';
}

/** SpreadsheetML 的工作表用默认命名空间，所以「命名空间为空」也算命中。 */
function isSpreadsheetElement(node: Node | null, localName: string): boolean {
  return isElementNode(node)
    && node.localName === localName
    && (node.namespaceURI === SPREADSHEET_NAMESPACE
      || node.namespaceURI === PACKAGE_RELATIONSHIPS_NAMESPACE
      || node.namespaceURI === null);
}

function isElementNode(node: Node | null): node is Element {
  return node !== null && node.nodeType === 1;
}

function childElements(node: Node): Element[] {
  const children: Element[] = [];
  for (let index = 0; index < node.childNodes.length; index += 1) {
    const child = node.childNodes.item(index);
    if (isElementNode(child)) {
      children.push(child);
    }
  }
  return children;
}

function findChildElement(node: Element, localName: string): Element | undefined {
  return childElements(node).find((child) => isSpreadsheetElement(child, localName));
}

function readChildElementText(node: Element, localName: string): string | undefined {
  const child = findChildElement(node, localName);
  return child ? child.textContent ?? '' : undefined;
}

/** 收集子树里所有同名元素（文档顺序即出现顺序，工作表顺序依赖它）。 */
function collectElements(root: Node, localName: string): Element[] {
  const elements: Element[] = [];
  for (const child of childElements(root)) {
    if (isSpreadsheetElement(child, localName)) {
      elements.push(child);
    }
    elements.push(...collectElements(child, localName));
  }
  return elements;
}

function findFirstElement(root: Node, localName: string): Element | undefined {
  for (const child of childElements(root)) {
    if (isSpreadsheetElement(child, localName)) {
      return child;
    }
    const found = findFirstElement(child, localName);
    if (found) {
      return found;
    }
  }
  return undefined;
}
