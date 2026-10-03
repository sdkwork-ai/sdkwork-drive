/* @vitest-environment jsdom */
import { describe, expect, it } from 'vitest';
import { parsePresentation } from '../../src/ooxml/presentationPreview';
import { buildZipArchive } from './zipFixture';

const SLIDE_NAMESPACE = 'xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"';

function titleShape(text: string, type: 'title' | 'ctrTitle'): string {
  return `<p:sp>
    <p:nvSpPr><p:cNvPr id="2" name="Title"/><p:cNvSpPr/><p:nvPr><p:ph type="${type}"/></p:nvPr></p:nvSpPr>
    <p:txBody><a:bodyPr/><a:p><a:r><a:t>${text}</a:t></a:r></a:p></p:txBody>
  </p:sp>`;
}

function bodyShape(paragraphs: string[]): string {
  const content = paragraphs.map((text) => `<a:p><a:r><a:t>${text}</a:t></a:r></a:p>`).join('');
  return `<p:sp>
    <p:nvSpPr><p:cNvPr id="3" name="Content"/><p:cNvSpPr/><p:nvPr><p:ph idx="1"/></p:nvPr></p:nvSpPr>
    <p:txBody><a:bodyPr/>${content}</p:txBody>
  </p:sp>`;
}

function slideXml(shapes: string): string {
  return `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld ${SLIDE_NAMESPACE}><p:cSld><p:spTree>${shapes}</p:spTree></p:cSld></p:sld>`;
}

function buildPresentation(parts: Record<string, string>): Uint8Array {
  return buildZipArchive(Object.entries(parts).map(([path, text]) => ({ path, text })));
}

function buildDefaultPresentation(): Uint8Array {
  return buildPresentation({
    // 归档顺序刻意打乱：期望的展示顺序必须来自数字编号而不是写入顺序。
    'ppt/slides/slide10.xml': slideXml(bodyShape(['正文行']) + titleShape('迟到的标题', 'title') + titleShape('第二个标题占位符', 'title')),
    'ppt/slides/slide2.xml': slideXml(
      titleShape('第二页标题', 'ctrTitle')
      + bodyShape(['  要点一  ', '要点二', '   '])
      + `<p:graphicFrame><a:t>表格里的字</a:t></p:graphicFrame>`,
    ),
    'ppt/slides/slide1.xml': slideXml(titleShape('第一页标题', 'title') + bodyShape(['甲', '乙'])),
    'ppt/slides/_rels/slide1.xml.rels': '<Relationships/>',
    'ppt/notesSlides/notesSlide1.xml': slideXml(bodyShape(['备注文字'])),
  });
}

describe('parsePresentation', () => {
  it('orders slides numerically rather than lexicographically', async () => {
    const model = await parsePresentation(buildDefaultPresentation());

    expect(model.slides.map((slide) => slide.index)).toEqual([1, 2, 10]);
  });

  it('takes the title from the first title or ctrTitle placeholder', async () => {
    const model = await parsePresentation(buildDefaultPresentation());

    expect(model.slides.map((slide) => slide.title)).toEqual(['第一页标题', '第二页标题', '迟到的标题']);
  });

  it('collects other shape text as trimmed, non-empty lines', async () => {
    const model = await parsePresentation(buildDefaultPresentation());

    expect(model.slides[0]?.lines).toEqual(['甲', '乙']);
    expect(model.slides[1]?.lines).toEqual(['要点一', '要点二']);
  });

  it('keeps title text out of the body lines and ignores later title placeholders', async () => {
    const model = await parsePresentation(buildDefaultPresentation());
    const last = model.slides[2];

    expect(last?.title).toBe('迟到的标题');
    expect(last?.lines).toEqual(['正文行']);
  });

  it('skips slide-number fields, empty paragraphs and non-shape graphics', async () => {
    const model = await parsePresentation(buildPresentation({
      'ppt/slides/slide1.xml': slideXml(
        titleShape('标题', 'title')
        + bodyShape(['正文'])
        + `<p:sp><p:nvSpPr><p:cNvPr id="4" name="SlideNumber"/><p:cNvSpPr/><p:nvPr><p:ph type="sldNum"/></p:nvPr></p:nvSpPr>
             <p:txBody><a:bodyPr/><a:p><a:fld id="{1}" type="slidenum"><a:t>3</a:t></a:fld></a:p></p:txBody></p:sp>`,
      ),
    }));

    expect(model.slides[0]?.lines).toEqual(['正文']);
  });

  it('ignores notes slides and relationship parts', async () => {
    const model = await parsePresentation(buildDefaultPresentation());

    expect(model.slides).toHaveLength(3);
    expect(model.slides.flatMap((slide) => slide.lines)).not.toContain('备注文字');
  });

  it('leaves the title undefined when no title placeholder exists', async () => {
    const model = await parsePresentation(buildPresentation({
      'ppt/slides/slide1.xml': slideXml(bodyShape(['只有正文'])),
    }));

    expect(model.slides[0]).toEqual({ index: 1, lines: ['只有正文'] });
  });

  it('skips a damaged slide part but keeps the others', async () => {
    const model = await parsePresentation(buildPresentation({
      'ppt/slides/slide1.xml': slideXml(titleShape('好的一页', 'title')),
      'ppt/slides/slide2.xml': '<p:sld><p:cSld>',
    }));

    expect(model.slides.map((slide) => slide.index)).toEqual([1]);
  });

  it('returns an empty model when the archive has no slide parts', async () => {
    const model = await parsePresentation(buildPresentation({ 'ppt/presentation.xml': '<p:presentation/>' }));

    expect(model.slides).toEqual([]);
  });
});
