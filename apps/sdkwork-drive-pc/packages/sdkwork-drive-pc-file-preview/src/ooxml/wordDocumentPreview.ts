/**
 * `.docx` 主文档部件到预览视图模型的转换。
 *
 * 只依赖 WordprocessingML 的主命名空间与几个稳定的元素（`w:body`/`w:p`/`w:r`/`w:t`），
 * 不解析样式表：预览要的是「标题层级、列表层级、粗斜下划线、表格按行铺开」这几个
 * 一眼可辨的信息，把 `styles.xml`、编号定义、主题字体都读进来只会让一次预览付出
 * 与收益不相称的解析成本。
 */

import { readZipArchive, type ZipArchive } from './zipArchive';

/** WordprocessingML 主命名空间。 */
const WORDPROCESSING_NAMESPACE = 'http://schemas.openxmlformats.org/wordprocessingml/2006/main';

/** Dublin Core 命名空间，`dc:title` 在这里。 */
const DUBLIN_CORE_NAMESPACE = 'http://purl.org/dc/elements/1.1/';

/** 标题样式：英文模板是 `Heading1`，中文模板是 `标题 1`，两种都要认。 */
const LATIN_HEADING_STYLE_PATTERN = /^Heading\s*(\d+)$/i;
const CHINESE_HEADING_STYLE_PATTERN = /^标题\s*(\d+)$/;

/** 标题层级上限，与 `WordParagraphPreview.level` 声明的 1-6 对齐。 */
const MAX_HEADING_LEVEL = 6;

/** 主文档部件；`document2.xml` 是少数转换工具写出的备用名。 */
const DOCUMENT_PART_PATHS = ['word/document.xml', 'word/document2.xml'] as const;

/** 元数据部件。 */
const CORE_PROPERTIES_PART_PATH = 'docProps/core.xml';

/**
 * 段落里需要继续下钻的容器元素：超链接、智能标记、修订插入、内容控件都会把正文
 * 包在中间一层，不下钻就会漏掉这些文字。
 */
const PARAGRAPH_CONTAINER_NAMES = new Set(['hyperlink', 'smartTag', 'ins', 'sdt', 'sdtContent', 'bdo', 'dir', 'moveTo', 'fldSimple']);

/**
 * 段落/运行里必须整棵跳过的子树。
 *
 * `pPr`/`rPr` 是属性容器；`del`/`moveFrom` 是被修订删除的内容，Word 不会显示；
 * `drawing`/`pict`/`object` 里的文本框自带一套 `w:p`，若下钻会把文本框文字混进
 * 当前运行的文本里，宁可先不展示文本框内容。
 */
const SKIPPED_SUBTREE_NAMES = new Set(['pPr', 'rPr', 'del', 'moveFrom', 'drawing', 'pict', 'object', 'txbxContent', 'tbl']);

/** 正文级别的内容控件容器，需要下钻才能找到段落。 */
const BODY_CONTAINER_NAMES = new Set(['sdt', 'sdtContent', 'customXml', 'ins']);

/** 运行文本里代表换行的元素。 */
const LINE_BREAK_NAMES = new Set(['br', 'cr']);

export interface WordRunPreview {
  text: string;
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  code?: boolean;
}

export interface WordParagraphPreview {
  kind: 'heading' | 'paragraph' | 'listItem';
  /** 标题层级 1-6；列表项则是 `w:ilvl` 的原始值（0 起）。 */
  level?: number;
  runs: WordRunPreview[];
  alignment?: 'left' | 'center' | 'right' | 'justify';
}

export interface WordDocumentPreviewModel {
  /** `docProps/core.xml` 的 `dc:title`。 */
  title?: string;
  paragraphs: WordParagraphPreview[];
}

export type WordDocumentPreviewErrorCode = 'dom-parser-unavailable' | 'document-xml-invalid';

/**
 * `.docx` 预览失败的错误类型。
 *
 * 单独建模而不是复用 `ZipArchiveError`：调用方对「运行时没有 DOM」与「归档损坏」
 * 的处理完全不同（前者是环境错误，后者可以提示文件本身有问题）。
 */
export class WordDocumentPreviewError extends Error {
  readonly code: WordDocumentPreviewErrorCode;

  constructor(code: WordDocumentPreviewErrorCode, message: string) {
    super(message);
    this.name = 'WordDocumentPreviewError';
    this.code = code;
  }
}

/** 解析 `.docx` 字节，产出按文档顺序排列的段落视图模型。 */
export async function parseWordDocument(bytes: Uint8Array): Promise<WordDocumentPreviewModel> {
  const archive = readZipArchive(bytes);
  const title = await readCoreTitle(archive);
  const xml = await readFirstAvailablePart(archive, DOCUMENT_PART_PATHS);

  if (xml === undefined) {
    // 缺少主文档部件时给一个空模型而不是抛错：预览的职责是「尽量显示」，
    // 调用方拿到空段落仍能渲染出「没有内容」的界面。
    return title === undefined ? { paragraphs: [] } : { title, paragraphs: [] };
  }

  const paragraphs = readBodyParagraphs(parseXmlDocument(xml, 'word/document.xml'));
  return title === undefined ? { paragraphs } : { title, paragraphs };
}

async function readCoreTitle(archive: ZipArchive): Promise<string | undefined> {
  const xml = await archive.readText(CORE_PROPERTIES_PART_PATH);
  if (xml === undefined) {
    return undefined;
  }
  // 元数据损坏不该让整篇文档预览失败，所以这里用可失败的解析并放弃标题。
  const document = tryParseXmlDocument(xml);
  if (!document) {
    return undefined;
  }
  const element = document.getElementsByTagName('dc:title').item(0)
    ?? document.getElementsByTagNameNS(DUBLIN_CORE_NAMESPACE, 'title').item(0);
  const title = element?.textContent?.trim();
  return title ? title : undefined;
}

async function readFirstAvailablePart(
  archive: ZipArchive,
  paths: readonly string[],
): Promise<string | undefined> {
  for (const path of paths) {
    const xml = await archive.readText(path);
    if (xml !== undefined) {
      return xml;
    }
  }
  return undefined;
}

function parseXmlDocument(xml: string, partPath: string): XMLDocument {
  const document = tryParseXmlDocument(xml);
  if (!document) {
    throw new WordDocumentPreviewError('document-xml-invalid', `${partPath} 不是可解析的 XML`);
  }
  return document;
}

/**
 * 用 DOMParser 解析 XML。
 *
 * 这里依赖宿主提供的 `DOMParser` 而不是自带解析器：实体、命名空间、CDATA 的边界条件
 * 太多，自己实现等于重写一个 XML 解析器；没有 DOM 的运行时（纯 Node 脚本）会明确报错，
 * 而不是悄悄把文档解析成空内容。
 */
function tryParseXmlDocument(xml: string): XMLDocument | undefined {
  if (typeof DOMParser !== 'function') {
    throw new WordDocumentPreviewError(
      'dom-parser-unavailable',
      '解析 Word 文档需要 DOMParser：请在浏览器或 jsdom 等带 DOM 的运行时中调用 parseWordDocument',
    );
  }
  const document = new DOMParser().parseFromString(xml, 'application/xml');
  return isParserErrorDocument(document) ? undefined : document;
}

function isParserErrorDocument(document: XMLDocument): boolean {
  const root = document.documentElement;
  return root !== null && root.localName === 'parsererror';
}

function readBodyParagraphs(document: XMLDocument): WordParagraphPreview[] {
  const body = findFirstWordElement(document, 'body');
  if (!body) {
    return [];
  }
  const paragraphs: WordParagraphPreview[] = [];
  collectBodyParagraphs(body, paragraphs);
  dropTrailingEmptyParagraph(paragraphs);
  return paragraphs;
}

function collectBodyParagraphs(node: Node, paragraphs: WordParagraphPreview[]): void {
  for (const child of childElements(node)) {
    if (isWordElement(child, 'p')) {
      paragraphs.push(readParagraph(child));
      continue;
    }
    if (isWordElement(child, 'tbl')) {
      paragraphs.push(...readTable(child));
      continue;
    }
    if (isWordElementIn(child, BODY_CONTAINER_NAMES)) {
      collectBodyParagraphs(child, paragraphs);
    }
  }
}

/**
 * 丢掉 Word 在正文末尾固定补出的空段落。
 *
 * 只丢最后一段、且只在它确实为空时丢：若文档本身就以空段落结尾（作者有意留白），
 * 这里少显示一段比多显示一段的观感损失小。
 */
function dropTrailingEmptyParagraph(paragraphs: WordParagraphPreview[]): void {
  const last = paragraphs.at(-1);
  if (last && last.kind === 'paragraph' && last.runs.length === 0) {
    paragraphs.pop();
  }
}

function readParagraph(element: Element): WordParagraphPreview {
  const properties = findChildWordElement(element, 'pPr');
  const styleId = readValAttribute(properties ? findChildWordElement(properties, 'pStyle') : undefined);
  const headingLevel = readHeadingLevel(styleId);

  const paragraph: WordParagraphPreview = { kind: 'paragraph', runs: readRunPreviews(element) };
  if (headingLevel !== undefined) {
    paragraph.kind = 'heading';
    paragraph.level = headingLevel;
  } else if (properties && findChildWordElement(properties, 'numPr')) {
    paragraph.kind = 'listItem';
    paragraph.level = readListLevel(properties);
  }

  const alignment = readAlignment(properties ? findChildWordElement(properties, 'jc') : undefined);
  if (alignment !== undefined) {
    paragraph.alignment = alignment;
  }
  return paragraph;
}

function readHeadingLevel(styleId: string | undefined): number | undefined {
  if (!styleId) {
    return undefined;
  }
  const trimmed = styleId.trim();
  const match = LATIN_HEADING_STYLE_PATTERN.exec(trimmed) ?? CHINESE_HEADING_STYLE_PATTERN.exec(trimmed);
  if (!match) {
    return undefined;
  }
  const level = Number.parseInt(match[1] ?? '', 10);
  if (!Number.isInteger(level) || level < 1) {
    return undefined;
  }
  return Math.min(level, MAX_HEADING_LEVEL);
}

/** 列表层级取 `w:ilvl` 原值（0 起），缺失时按第 0 层处理。 */
function readListLevel(properties: Element): number {
  const numbering = findChildWordElement(properties, 'numPr');
  const level = readValAttribute(numbering ? findChildWordElement(numbering, 'ilvl') : undefined);
  const parsed = level === undefined ? Number.NaN : Number.parseInt(level, 10);
  return Number.isInteger(parsed) && parsed > 0 ? parsed : 0;
}

function readAlignment(element: Element | undefined): WordParagraphPreview['alignment'] {
  const value = readValAttribute(element)?.trim().toLowerCase();
  switch (value) {
    case 'left':
    case 'start':
      return 'left';
    case 'center':
      return 'center';
    case 'right':
    case 'end':
      return 'right';
    case 'both':
    case 'justify':
    case 'distribute':
      return 'justify';
    default:
      return undefined;
  }
}

/**
 * 表格铺平成段落：每行一段，单元格之间用 ` | ` 连接。
 *
 * 预览视图模型里没有表格结构，铺平成文本至少保证内容不丢、行序与列序可读。
 */
function readTable(table: Element): WordParagraphPreview[] {
  const paragraphs: WordParagraphPreview[] = [];
  for (const row of childElements(table)) {
    if (!isWordElement(row, 'tr')) {
      continue;
    }
    const cells: string[] = [];
    for (const cell of childElements(row)) {
      if (isWordElement(cell, 'tc')) {
        cells.push(readCellText(cell));
      }
    }
    paragraphs.push({ kind: 'paragraph', runs: cells.length > 0 ? [{ text: cells.join(' | ') }] : [] });
  }
  return paragraphs;
}

function readCellText(cell: Element): string {
  const parts: string[] = [];
  for (const paragraph of childElements(cell)) {
    if (!isWordElement(paragraph, 'p')) {
      continue;
    }
    const text = readRunPreviews(paragraph).map((run) => run.text).join('');
    if (text.length > 0) {
      parts.push(text);
    }
  }
  return parts.join(' ');
}

function readRunPreviews(paragraph: Element): WordRunPreview[] {
  const runs: WordRunPreview[] = [];
  collectRuns(paragraph, runs);
  return runs;
}

function collectRuns(node: Element, runs: WordRunPreview[]): void {
  for (const child of childElements(node)) {
    if (isWordElement(child, 'r')) {
      const run = readRun(child);
      // 空运行（只有属性没有文字）在视图模型里没有意义，还会让「段落是否为空」的
      // 判断变复杂。
      if (run.text.length > 0) {
        runs.push(run);
      }
      continue;
    }
    if (isWordElementIn(child, PARAGRAPH_CONTAINER_NAMES)) {
      collectRuns(child, runs);
    }
  }
}

function readRun(element: Element): WordRunPreview {
  const run: WordRunPreview = { text: readRunText(element) };
  const properties = findChildWordElement(element, 'rPr');
  if (properties) {
    applyRunFlags(properties, run);
  }
  return run;
}

/**
 * 按文档顺序取出运行文本，`w:tab`/`w:br` 转成制表符与换行。
 *
 * `xml:space="preserve"` 不需要额外处理：文档按 `application/xml` 解析，XML 解析器
 * 本来就不折叠空白，直接取 `textContent` 就是原样文本。
 */
function readRunText(node: Element): string {
  let text = '';
  for (const child of childElements(node)) {
    if (isWordElementIn(child, SKIPPED_SUBTREE_NAMES)) {
      continue;
    }
    if (isWordElement(child, 't')) {
      text += child.textContent ?? '';
      continue;
    }
    if (isWordElement(child, 'tab')) {
      text += '\t';
      continue;
    }
    if (isWordElementIn(child, LINE_BREAK_NAMES)) {
      text += '\n';
      continue;
    }
    text += readRunText(child);
  }
  return text;
}

function applyRunFlags(properties: Element, run: WordRunPreview): void {
  if (isToggleOn(findChildWordElement(properties, 'b'))) {
    run.bold = true;
  }
  if (isToggleOn(findChildWordElement(properties, 'i'))) {
    run.italic = true;
  }
  const underlineElement = findChildWordElement(properties, 'u');
  const underline = readValAttribute(underlineElement)?.trim().toLowerCase();
  // `<w:u/>` 没有 val 就是单下划线；只有显式 `none` 才是关闭。
  if (underlineElement && underline !== 'none') {
    run.underline = true;
  }
  const styleId = readValAttribute(findChildWordElement(properties, 'rStyle'));
  if (styleId !== undefined && /code$/i.test(styleId.trim())) {
    run.code = true;
  }
}

/**
 * 开关型属性的取值语义：没有 `w:val` 视为开启，`0`/`false`/`off` 视为关闭。
 *
 * Word 里 `<w:b/>` 与 `<w:b w:val="true"/>` 等价，而 `<w:b w:val="0"/>` 常用于
 * 在粗体样式里显式关掉粗体，只看元素是否存在会判反。
 */
function isToggleOn(element: Element | undefined): boolean {
  if (!element) {
    return false;
  }
  const value = readValAttribute(element)?.trim().toLowerCase();
  return value === undefined || (value !== '0' && value !== 'false' && value !== 'off');
}

function readValAttribute(element: Element | undefined): string | undefined {
  if (!element) {
    return undefined;
  }
  const value = element.getAttribute('w:val') ?? element.getAttribute('val');
  return value === null ? undefined : value;
}

/** 按限定名匹配 WordprocessingML 元素；前缀不是权威，命名空间才是。 */
function isWordElement(node: Node | null, localName: string): boolean {
  return isElementNode(node)
    && node.localName === localName
    && (node.namespaceURI === WORDPROCESSING_NAMESPACE || node.prefix === 'w');
}

function isWordElementIn(node: Node | null, names: ReadonlySet<string>): boolean {
  return isElementNode(node)
    && names.has(node.localName ?? '')
    && (node.namespaceURI === WORDPROCESSING_NAMESPACE || node.prefix === 'w');
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

function findChildWordElement(node: Element, localName: string): Element | undefined {
  return childElements(node).find((child) => isWordElement(child, localName));
}

function findFirstWordElement(root: Node, localName: string): Element | undefined {
  for (const child of childElements(root)) {
    if (isWordElement(child, localName)) {
      return child;
    }
    const found = findFirstWordElement(child, localName);
    if (found) {
      return found;
    }
  }
  return undefined;
}
