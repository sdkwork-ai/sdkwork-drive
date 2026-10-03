/**
 * `.pptx` 到幻灯片大纲视图模型的转换。
 *
 * 取的是「每页标题 + 正文行」这一层信息：形状的位置、主题字体、动画都在
 * `p:sldLayout`/`p:sldMaster`/主题部件里，对预览没有增量价值，反而要求先做一遍
 * 占位符继承解析。因此这里只认幻灯片自身 XML 里的静态文本。
 */

import { readZipArchive, type ZipArchive } from './zipArchive';

/** PresentationML 主命名空间（幻灯片部件里的 `p:` 前缀）。 */
const PRESENTATION_NAMESPACE = 'http://schemas.openxmlformats.org/presentationml/2006/main';

/** DrawingML 主命名空间，形状文本 `a:t` 在这里。 */
const DRAWING_NAMESPACE = 'http://schemas.openxmlformats.org/drawingml/2006/main';

/** 幻灯片部件：只匹配 `ppt/slides/slideN.xml`，不匹配 `notesSlides` 与 `_rels`。 */
const SLIDE_PART_PATTERN = /^ppt\/slides\/slide(\d+)\.xml$/;

/** 标题占位符的类型值。 */
const TITLE_PLACEHOLDER_TYPES = new Set(['title', 'ctrTitle']);

/**
 * 整棵跳过的子树：`a:fld` 是页码/日期这类字段的缓存文本，它属于版式而不是内容，
 * 显示在正文里会变成一行莫名其妙的数字。
 */
const SKIPPED_TEXT_SUBTREE_NAMES = new Set(['fld']);

export interface SlidePreview {
  /** 幻灯片编号，取自 `slideN.xml` 的 N（1 起，与 PowerPoint 的页码一致）。 */
  index: number;
  title?: string;
  lines: string[];
}

export interface PresentationPreviewModel {
  slides: SlidePreview[];
}

export type PresentationPreviewErrorCode = 'dom-parser-unavailable';

/** `.pptx` 预览失败的错误类型。 */
export class PresentationPreviewError extends Error {
  readonly code: PresentationPreviewErrorCode;

  constructor(code: PresentationPreviewErrorCode, message: string) {
    super(message);
    this.name = 'PresentationPreviewError';
    this.code = code;
  }
}

/** 解析 `.pptx` 字节，按幻灯片编号升序产出大纲视图模型。 */
export async function parsePresentation(bytes: Uint8Array): Promise<PresentationPreviewModel> {
  const archive = readZipArchive(bytes);
  const slides: SlidePreview[] = [];

  for (const slidePart of collectSlideParts(archive)) {
    const xml = await archive.readText(slidePart.path);
    if (xml === undefined) {
      continue;
    }
    const document = tryParsePresentationXml(xml);
    if (!document) {
      // 单页 XML 损坏时保留其余幻灯片，总比整份演示文稿打不开好。
      continue;
    }
    slides.push(readSlidePreview(slidePart.index, document));
  }

  return { slides };
}

/**
 * 收集幻灯片部件并按编号排序。
 *
 * 不能按路径字符串排序：字典序会把 `slide10.xml` 排到 `slide2.xml` 前面，
 * 而 PowerPoint 的顺序是数字顺序。
 */
function collectSlideParts(archive: ZipArchive): { index: number; path: string }[] {
  const parts: { index: number; path: string }[] = [];
  for (const entry of archive.entries) {
    if (entry.isDirectory) {
      continue;
    }
    const match = SLIDE_PART_PATTERN.exec(entry.path);
    const index = match ? Number.parseInt(match[1] ?? '', 10) : Number.NaN;
    if (Number.isInteger(index)) {
      parts.push({ index, path: entry.path });
    }
  }
  return parts.sort((left, right) => left.index - right.index);
}

function readSlidePreview(index: number, document: XMLDocument): SlidePreview {
  let title: string | undefined;
  const lines: string[] = [];

  for (const shape of collectElements(document, 'sp')) {
    const isTitle = isTitlePlaceholder(shape);
    if (isTitle && title === undefined) {
      const candidate = readShapeText(shape).join(' ').trim();
      if (candidate.length > 0) {
        title = candidate;
      }
      continue;
    }
    // 占位符是标题的形状整体不计入正文：否则同一段文字会在标题与正文里各出现一次。
    if (isTitle) {
      continue;
    }
    for (const item of readShapeText(shape)) {
      const line = item.trim();
      if (line.length > 0) {
        lines.push(line);
      }
    }
  }

  return title === undefined ? { index, lines } : { index, title, lines };
}

function isTitlePlaceholder(shape: Element): boolean {
  const nonVisualProperties = findChildPath(shape, ['nvSpPr', 'nvPr']);
  const placeholder = nonVisualProperties ? findChildElement(nonVisualProperties, 'ph') : undefined;
  const type = placeholder?.getAttribute('type');
  return type !== null && type !== undefined && TITLE_PLACEHOLDER_TYPES.has(type);
}

/** 按文档顺序取出形状内所有 `a:t` 文本，一个元素对应一行。 */
function readShapeText(shape: Element): string[] {
  const parts: string[] = [];
  collectShapeText(shape, parts);
  return parts;
}

function collectShapeText(node: Element, parts: string[]): void {
  for (const child of childElements(node)) {
    if (isDrawingElementIn(child, SKIPPED_TEXT_SUBTREE_NAMES)) {
      continue;
    }
    if (isDrawingElement(child, 't')) {
      parts.push(child.textContent ?? '');
      continue;
    }
    collectShapeText(child, parts);
  }
}

function tryParsePresentationXml(xml: string): XMLDocument | undefined {
  if (typeof DOMParser !== 'function') {
    throw new PresentationPreviewError(
      'dom-parser-unavailable',
      '解析 PowerPoint 演示文稿需要 DOMParser：请在浏览器或 jsdom 等带 DOM 的运行时中调用 parsePresentation',
    );
  }
  const document = new DOMParser().parseFromString(xml, 'application/xml');
  return isParserErrorDocument(document) ? undefined : document;
}

function isParserErrorDocument(document: XMLDocument): boolean {
  const root = document.documentElement;
  return root !== null && root.localName === 'parsererror';
}

function isPresentationElement(node: Node | null, localName: string): boolean {
  return isElementNode(node)
    && node.localName === localName
    && (node.namespaceURI === PRESENTATION_NAMESPACE || node.prefix === 'p');
}

function isDrawingElement(node: Node | null, localName: string): boolean {
  return isElementNode(node)
    && node.localName === localName
    && (node.namespaceURI === DRAWING_NAMESPACE || node.prefix === 'a');
}

function isDrawingElementIn(node: Node | null, names: ReadonlySet<string>): boolean {
  return isElementNode(node)
    && names.has(node.localName ?? '')
    && (node.namespaceURI === DRAWING_NAMESPACE || node.prefix === 'a');
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
  return childElements(node).find((child) => isPresentationElement(child, localName));
}

/** 按固定层级逐层向下找，例如 `p:sp > p:nvSpPr > p:nvPr`。 */
function findChildPath(node: Element, localNames: readonly string[]): Element | undefined {
  let current: Element | undefined = node;
  for (const localName of localNames) {
    if (!current) {
      return undefined;
    }
    current = findChildElement(current, localName);
  }
  return current;
}

/** 收集子树里所有 PresentationML 同名元素，文档顺序即形状顺序。 */
function collectElements(root: Node, localName: string): Element[] {
  const elements: Element[] = [];
  for (const child of childElements(root)) {
    if (isPresentationElement(child, localName)) {
      elements.push(child);
    }
    elements.push(...collectElements(child, localName));
  }
  return elements;
}
