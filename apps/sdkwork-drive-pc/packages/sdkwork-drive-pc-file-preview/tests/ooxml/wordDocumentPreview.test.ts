/* @vitest-environment jsdom */
import { describe, expect, it } from 'vitest';
import {
  WordDocumentPreviewError,
  parseWordDocument,
} from '../../src/ooxml/wordDocumentPreview';
import { buildZipArchive } from './zipFixture';

const DOCUMENT_XML = `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>季度报告</w:t></w:r></w:p>
    <w:p><w:pPr><w:pStyle w:val="标题 2"/></w:pPr><w:r><w:t>中文标题</w:t></w:r></w:p>
    <w:p>
      <w:pPr><w:numPr><w:ilvl w:val="1"/><w:numId w:val="1"/></w:numPr></w:pPr>
      <w:r><w:t>第二层列表项</w:t></w:r>
    </w:p>
    <w:p>
      <w:r><w:rPr><w:b/></w:rPr><w:t>粗体</w:t></w:r>
      <w:r><w:rPr><w:i w:val="true"/></w:rPr><w:t>斜体</w:t></w:r>
      <w:r><w:rPr><w:u w:val="single"/></w:rPr><w:t>下划线</w:t></w:r>
      <w:r><w:rPr><w:rStyle w:val="SourceCode"/></w:rPr><w:t>const a = 1;</w:t></w:r>
      <w:r><w:rPr><w:b w:val="0"/></w:rPr><w:t>非粗体</w:t></w:r>
      <w:r><w:t xml:space="preserve"> 保留空格 </w:t></w:r>
      <w:r><w:t>前</w:t><w:tab/><w:t>后</w:t><w:br/><w:t>换行</w:t></w:r>
    </w:p>
    <w:p><w:pPr><w:jc w:val="both"/></w:pPr><w:r><w:t>两端对齐</w:t></w:r></w:p>
    <w:tbl>
      <w:tr><w:tc><w:p><w:r><w:t>姓名</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>年龄</w:t></w:r></w:p></w:tc></w:tr>
      <w:tr><w:tc><w:p><w:r><w:t>张三</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>30</w:t></w:r></w:p></w:tc></w:tr>
    </w:tbl>
    <w:p/>
  </w:body>
</w:document>`;

const CORE_XML = `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:title>2026 年度报告</dc:title>
</cp:coreProperties>`;

function buildDocx(parts: Record<string, string>): Uint8Array {
  return buildZipArchive(Object.entries(parts).map(([path, text]) => ({ path, text })));
}

function buildDefaultDocx(): Uint8Array {
  return buildDocx({
    '[Content_Types].xml': '<Types/>',
    'docProps/core.xml': CORE_XML,
    'word/document.xml': DOCUMENT_XML,
  });
}

describe('parseWordDocument', () => {
  it('reads the Dublin Core title from docProps/core.xml', async () => {
    const model = await parseWordDocument(buildDefaultDocx());

    expect(model.title).toBe('2026 年度报告');
  });

  it('walks body children in document order', async () => {
    const model = await parseWordDocument(buildDefaultDocx());

    expect(model.paragraphs.map((paragraph) => paragraph.kind)).toEqual([
      'heading',
      'heading',
      'listItem',
      'paragraph',
      'paragraph',
      'paragraph',
      'paragraph',
    ]);
  });

  it('maps pStyle to heading levels for English and Chinese styles', async () => {
    const model = await parseWordDocument(buildDefaultDocx());

    expect(model.paragraphs[0]).toMatchObject({ kind: 'heading', level: 1 });
    expect(model.paragraphs[1]).toMatchObject({ kind: 'heading', level: 2 });
  });

  it('maps numPr to a list item and takes the level from w:ilvl', async () => {
    const model = await parseWordDocument(buildDefaultDocx());

    expect(model.paragraphs[2]).toEqual({
      kind: 'listItem',
      level: 1,
      runs: [{ text: '第二层列表项' }],
    });
  });

  it('maps run properties to bold/italic/underline/code flags', async () => {
    const model = await parseWordDocument(buildDefaultDocx());

    expect(model.paragraphs[3]?.runs).toEqual([
      { text: '粗体', bold: true },
      { text: '斜体', italic: true },
      { text: '下划线', underline: true },
      { text: 'const a = 1;', code: true },
      { text: '非粗体' },
      { text: ' 保留空格 ' },
      { text: '前\t后\n换行' },
    ]);
  });

  it('maps w:jc to the alignment field', async () => {
    const model = await parseWordDocument(buildDefaultDocx());

    expect(model.paragraphs[4]).toMatchObject({ alignment: 'justify' });
  });

  it('flattens tables into one paragraph per row joined with a pipe', async () => {
    const model = await parseWordDocument(buildDefaultDocx());

    expect(model.paragraphs[5]).toEqual({ kind: 'paragraph', runs: [{ text: '姓名 | 年龄' }] });
    expect(model.paragraphs[6]).toEqual({ kind: 'paragraph', runs: [{ text: '张三 | 30' }] });
  });

  it('drops the trailing empty paragraph Word always writes', async () => {
    const model = await parseWordDocument(buildDefaultDocx());

    expect(model.paragraphs).toHaveLength(7);
  });

  it('keeps a trailing paragraph that has text', async () => {
    const model = await parseWordDocument(buildDocx({
      'word/document.xml': `<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>
        <w:p><w:r><w:t>正文</w:t></w:r></w:p>
        <w:p><w:r><w:t>结尾</w:t></w:r></w:p>
      </w:body></w:document>`,
    }));

    expect(model.paragraphs).toHaveLength(2);
    expect(model.paragraphs[1]?.runs).toEqual([{ text: '结尾' }]);
  });

  it('keeps inserted and hyperlinked runs but drops deleted revision text', async () => {
    const model = await parseWordDocument(buildDocx({
      'word/document.xml': `<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>
        <w:p>
          <w:del><w:r><w:delText>删除的内容</w:delText></w:r></w:del>
          <w:ins><w:r><w:t>插入的内容</w:t></w:r></w:ins>
          <w:hyperlink r:id="rId1" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:r><w:t>链接文字</w:t></w:r></w:hyperlink>
        </w:p>
      </w:body></w:document>`,
    }));

    expect(model.paragraphs[0]?.runs).toEqual([{ text: '插入的内容' }, { text: '链接文字' }]);
  });

  it('clamps heading levels to the supported 1-6 range', async () => {
    const model = await parseWordDocument(buildDocx({
      'word/document.xml': `<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>
        <w:p><w:pPr><w:pStyle w:val="Heading9"/></w:pPr><w:r><w:t>很深的标题</w:t></w:r></w:p>
        <w:p><w:r><w:t>结尾</w:t></w:r></w:p>
      </w:body></w:document>`,
    }));

    expect(model.paragraphs[0]).toMatchObject({ kind: 'heading', level: 6 });
  });

  it('falls back to word/document2.xml when the main part is absent', async () => {
    const model = await parseWordDocument(buildDocx({
      'word/document2.xml': `<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>
        <w:p><w:r><w:t>备用部件</w:t></w:r></w:p>
      </w:body></w:document>`,
    }));

    expect(model.paragraphs).toEqual([{ kind: 'paragraph', runs: [{ text: '备用部件' }] }]);
  });

  it('returns an empty model when the archive has no document part', async () => {
    const model = await parseWordDocument(buildDocx({ '[Content_Types].xml': '<Types/>' }));

    expect(model).toEqual({ paragraphs: [] });
  });

  it('ignores a corrupt core properties part but still reads the body', async () => {
    const model = await parseWordDocument(buildDocx({
      'docProps/core.xml': '<cp:coreProperties><dc:title>坏掉的',
      'word/document.xml': `<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>
        <w:p><w:r><w:t>正文</w:t></w:r></w:p>
      </w:body></w:document>`,
    }));

    expect(model.title).toBeUndefined();
    expect(model.paragraphs).toEqual([{ kind: 'paragraph', runs: [{ text: '正文' }] }]);
  });

  it('reads a deflate-compressed document part', async () => {
    // 真实 Word 归档里的 document.xml 是 deflate 压缩的；jsdom 的 Blob 没有 stream()，
    // 这条用例同时守住「没有 Blob.stream 时仍能解压」的退路。
    const model = await parseWordDocument(buildZipArchive([
      { path: 'docProps/core.xml', text: CORE_XML, method: 8 },
      { path: 'word/document.xml', text: DOCUMENT_XML, method: 8 },
    ]));

    expect(model.title).toBe('2026 年度报告');
    expect(model.paragraphs[0]).toMatchObject({ kind: 'heading', level: 1 });
  });

  it('fails with a typed error when the runtime has no DOMParser', async () => {    const descriptor = Object.getOwnPropertyDescriptor(globalThis, 'DOMParser');
    if (!descriptor || descriptor.configurable !== true) {
      return;
    }

    Reflect.deleteProperty(globalThis, 'DOMParser');
    try {
      await expect(parseWordDocument(buildDefaultDocx())).rejects.toMatchObject({
        name: 'WordDocumentPreviewError',
        code: 'dom-parser-unavailable',
      });
    } finally {
      Object.defineProperty(globalThis, 'DOMParser', descriptor);
    }
    expect(new WordDocumentPreviewError('dom-parser-unavailable', 'x')).toBeInstanceOf(Error);
  });
});
