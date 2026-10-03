/* @vitest-environment jsdom */
import { describe, expect, it } from 'vitest';
import {
  MAX_SPREADSHEET_PREVIEW_COLUMNS,
  MAX_SPREADSHEET_PREVIEW_ROWS,
  parseCsvSpreadsheet,
  parseSpreadsheet,
} from '../../src/ooxml/spreadsheetPreview';
import { buildZipArchive, textBytes } from './zipFixture';

const WORKBOOK_XML = `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <bookViews><workbookView activeTab="1"/></bookViews>
  <sheets>
    <sheet name="汇总" sheetId="1" r:id="rId1"/>
    <sheet name="明细" sheetId="2" r:id="rId2"/>
  </sheets>
</workbook>`;

const WORKBOOK_RELS_XML = `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="/xl/worksheets/sheet2.xml"/>
  <Relationship Id="rId9" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="https://example.com" TargetMode="External"/>
</Relationships>`;

const SHARED_STRINGS_XML = `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="4" uniqueCount="4">
  <si><t>产品</t></si>
  <si><r><t>富</t></r><r><rPr><b/></rPr><t>文本</t></r></si>
  <si><t xml:space="preserve"> 带空格 </t></si>
  <si><rPh sb="0" eb="1"><t>zhu</t></rPh><t>注音</t></si>
</sst>`;

const SHEET_ONE_XML = `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData>
    <row r="1">
      <c r="A1" t="s"><v>0</v></c>
      <c r="B1" t="s"><v>1</v></c>
      <c r="C1" t="inlineStr"><is><t>内联</t></is></c>
    </row>
    <row r="2">
      <c r="A2"><v>10</v></c>
      <c r="C2" t="n"><v>30</v></c>
      <c r="D2" t="s"><v>99</v></c>
    </row>
    <row r="3" hidden="1"><c r="A3" t="s"><v>2</v></c></row>
    <row r="4"><c r="A4" t="s"><v>3</v></c></row>
  </sheetData>
</worksheet>`;

const SHEET_TWO_XML = `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData><row r="1"><c r="A1" t="b"><v>1</v></c></row></sheetData>
</worksheet>`;

function buildWorkbook(parts: Record<string, string>): Uint8Array {
  return buildZipArchive(Object.entries(parts).map(([path, text]) => ({ path, text })));
}

function buildDefaultWorkbook(): Uint8Array {
  return buildWorkbook({
    'xl/workbook.xml': WORKBOOK_XML,
    'xl/_rels/workbook.xml.rels': WORKBOOK_RELS_XML,
    'xl/sharedStrings.xml': SHARED_STRINGS_XML,
    'xl/worksheets/sheet1.xml': SHEET_ONE_XML,
    'xl/worksheets/sheet2.xml': SHEET_TWO_XML,
  });
}

function textsOf(row: { text: string }[] | undefined): string[] {
  return (row ?? []).map((cell) => cell.text);
}

/** 生成列名：0 → A，25 → Z，26 → AA。 */
function columnName(index: number): string {
  let name = '';
  let current = index + 1;
  while (current > 0) {
    const remainder = (current - 1) % 26;
    name = String.fromCharCode(65 + remainder) + name;
    current = Math.floor((current - 1) / 26);
  }
  return name;
}

describe('parseSpreadsheet (xlsx)', () => {
  it('reads sheet names in workbook order and resolves their parts through the rels', async () => {
    const model = await parseSpreadsheet(buildDefaultWorkbook(), 'book.xlsx');

    expect(model.sheets.map((sheet) => sheet.name)).toEqual(['汇总', '明细']);
    expect(textsOf(model.sheets[1]?.rows[0])).toEqual(['1']);
  });

  it('takes the active sheet index from workbookView', async () => {
    const model = await parseSpreadsheet(buildDefaultWorkbook(), 'book.xlsx');

    expect(model.activeSheetIndex).toBe(1);
  });

  it('resolves shared strings, including concatenated rich-text runs', async () => {
    const model = await parseSpreadsheet(buildDefaultWorkbook(), 'book.xlsx');

    expect(textsOf(model.sheets[0]?.rows[0])).toEqual(['产品', '富文本', '内联']);
  });

  it('drops phonetic hints and keeps xml:space content of shared strings', async () => {
    const model = await parseSpreadsheet(buildDefaultWorkbook(), 'book.xlsx');

    expect(textsOf(model.sheets[0]?.rows[2])).toEqual([' 带空格 ']);
    expect(textsOf(model.sheets[0]?.rows[3])).toEqual(['注音']);
  });

  it('keeps every row, including rows marked hidden', async () => {
    const model = await parseSpreadsheet(buildDefaultWorkbook(), 'book.xlsx');

    expect(model.sheets[0]?.rows).toHaveLength(4);
  });

  it('marks the first row as the header row only', async () => {
    const model = await parseSpreadsheet(buildDefaultWorkbook(), 'book.xlsx');
    const rows = model.sheets[0]?.rows ?? [];

    expect(rows[0]?.every((cell) => cell.isHeader)).toBe(true);
    expect(rows[1]?.some((cell) => cell.isHeader)).toBe(false);
  });

  it('aligns sparse cells by the column letters in the cell reference', async () => {
    const model = await parseSpreadsheet(buildDefaultWorkbook(), 'book.xlsx');
    const row = model.sheets[0]?.rows[1] ?? [];

    // B2 缺失，C2 仍必须落在第 2 列。
    expect(textsOf(row)).toEqual(['10', '', '30', '']);
    expect(row[0]?.isNumeric).toBe(true);
    expect(row[1]?.isNumeric).toBe(false);
    expect(row[2]?.isNumeric).toBe(true);
  });

  it('treats a shared-string index outside the table as empty text', async () => {
    const model = await parseSpreadsheet(buildDefaultWorkbook(), 'book.xlsx');

    expect(textsOf(model.sheets[0]?.rows[1])[3]).toBe('');
  });

  it('renders t="b" cells as text rather than numbers', async () => {
    const model = await parseSpreadsheet(buildDefaultWorkbook(), 'book.xlsx');
    const cell = model.sheets[1]?.rows[0]?.[0];

    expect(cell?.text).toBe('1');
    expect(cell?.isNumeric).toBe(false);
  });

  it('truncates rows beyond the preview limit and flags it', async () => {
    const rowCount = MAX_SPREADSHEET_PREVIEW_ROWS + 1;
    const rows = Array.from(
      { length: rowCount },
      (_, index) => `<row r="${index + 1}"><c r="A${index + 1}"><v>${index}</v></c></row>`,
    ).join('');
    const model = await parseSpreadsheet(buildWorkbook({
      'xl/workbook.xml': `<?xml version="1.0"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheets><sheet name="长表" sheetId="1" r:id="rId1" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"/></sheets></workbook>`,
      'xl/worksheets/sheet1.xml': `<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>${rows}</sheetData></worksheet>`,
    }), 'long.xlsx');

    expect(model.sheets[0]?.rows).toHaveLength(MAX_SPREADSHEET_PREVIEW_ROWS);
    expect(model.sheets[0]?.truncatedRows).toBe(true);
    expect(model.sheets[0]?.rows.at(-1)?.[0]?.text).toBe(String(MAX_SPREADSHEET_PREVIEW_ROWS - 1));
  });

  it('truncates columns beyond the preview limit', async () => {
    const cells = Array.from(
      { length: MAX_SPREADSHEET_PREVIEW_COLUMNS + 6 },
      (_, index) => `<c r="${columnName(index)}1"><v>${index}</v></c>`,
    ).join('');
    const model = await parseSpreadsheet(buildWorkbook({
      'xl/workbook.xml': `<?xml version="1.0"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheets><sheet name="宽表" sheetId="1" r:id="rId1" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"/></sheets></workbook>`,
      'xl/worksheets/sheet1.xml': `<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1">${cells}</row></sheetData></worksheet>`,
    }), 'wide.xlsx');

    const row = model.sheets[0]?.rows[0] ?? [];
    expect(row).toHaveLength(MAX_SPREADSHEET_PREVIEW_COLUMNS);
    // 第 27 列（AA）必须按双字母解析，而不是被当成第 0 列。
    expect(row[26]?.text).toBe('26');
    expect(model.sheets[0]?.truncatedRows).toBe(false);
    // 列被截断时行没有被截断；两个标志互相独立，界面才能分别说清少了什么。
    expect(model.sheets[0]?.truncatedColumns).toBe(true);
  });
});

describe('parseCsvSpreadsheet', () => {
  it('parses plain rows and marks the header row', () => {
    const model = parseCsvSpreadsheet('name,qty\r\napple,3\r\n');
    const sheet = model.sheets[0];

    expect(sheet?.name).toBe('Sheet1');
    expect(sheet?.truncatedRows).toBe(false);
    expect(textsOf(sheet?.rows[0])).toEqual(['name', 'qty']);
    expect(textsOf(sheet?.rows[1])).toEqual(['apple', '3']);
    expect(sheet?.rows[0]?.[0]?.isHeader).toBe(true);
    expect(sheet?.rows[1]?.[1]?.isHeader).toBe(false);
    expect(sheet?.rows[1]?.[1]?.isNumeric).toBe(true);
  });

  it('honours RFC 4180 quoting, escaped quotes and embedded newlines', () => {
    const model = parseCsvSpreadsheet('"a,b",c\r\n"line1\nline2","say ""hi"""\r\n');
    const rows = model.sheets[0]?.rows ?? [];

    expect(textsOf(rows[0])).toEqual(['a,b', 'c']);
    expect(textsOf(rows[1])).toEqual(['line1\nline2', 'say "hi"']);
  });

  it('treats lone CR as a row separator and ignores a trailing newline', () => {
    const model = parseCsvSpreadsheet('a,b\r1,2\r');

    expect(model.sheets[0]?.rows).toHaveLength(2);
    expect(textsOf(model.sheets[0]?.rows[1])).toEqual(['1', '2']);
  });

  it('returns no rows for empty input and one empty row for a single newline', () => {
    expect(parseCsvSpreadsheet('').sheets[0]?.rows).toEqual([]);
    // RFC 4180 里一行结束就是一条记录，因此「只有一个换行」是含一个空字段的记录。
    expect(textsOf(parseCsvSpreadsheet('\n').sheets[0]?.rows[0])).toEqual(['']);
  });

  it('uses a strict numeric check', () => {
    const model = parseCsvSpreadsheet('1,-1.5,1e3,.5,+2,007,"1,234",12abc,,abc');
    const flags = (model.sheets[0]?.rows[0] ?? []).map((cell) => cell.isNumeric);

    expect(flags).toEqual([true, true, true, true, true, true, false, false, false, false]);
  });

  it('accepts an explicit sheet name', () => {
    expect(parseCsvSpreadsheet('a', '订单').sheets[0]?.name).toBe('订单');
  });

  it('truncates rows beyond the preview limit and flags it', () => {
    const lineCount = MAX_SPREADSHEET_PREVIEW_ROWS + 10;
    const csv = Array.from({ length: lineCount }, (_, index) => `row${index},${index}`).join('\n');
    const sheet = parseCsvSpreadsheet(csv).sheets[0];

    expect(sheet?.rows).toHaveLength(MAX_SPREADSHEET_PREVIEW_ROWS);
    expect(sheet?.truncatedRows).toBe(true);
  });

  it('does not flag truncation when the row count is exactly at the limit', () => {
    const csv = Array.from({ length: MAX_SPREADSHEET_PREVIEW_ROWS }, (_, index) => `row${index}`).join('\n');
    const sheet = parseCsvSpreadsheet(csv).sheets[0];

    expect(sheet?.rows).toHaveLength(MAX_SPREADSHEET_PREVIEW_ROWS);
    expect(sheet?.truncatedRows).toBe(false);
  });
});

describe('parseSpreadsheet (delimited text dispatch)', () => {
  it('splits .tsv on tabs instead of commas', async () => {
    // 制表符分隔的文件里值本身常含逗号；按逗号切会把整行挤成一列。
    const model = await parseSpreadsheet(textBytes('name\tnote\nAda\t"a, b"\n'), 'export.tsv');

    expect(model.sheets[0]?.name).toBe('export');
    expect(textsOf(model.sheets[0]?.rows[0])).toEqual(['name', 'note']);
    expect(textsOf(model.sheets[0]?.rows[1])).toEqual(['Ada', 'a, b']);
  });

  it('sniffs a semicolon-delimited .csv', async () => {
    const model = await parseSpreadsheet(textBytes('name;amount\nAda;10\n'), 'regional.csv');

    expect(textsOf(model.sheets[0]?.rows[0])).toEqual(['name', 'amount']);
    expect(textsOf(model.sheets[0]?.rows[1])).toEqual(['Ada', '10']);
  });
});

describe('parseSpreadsheet (csv dispatch)', () => {
  it('routes .csv payloads through the CSV parser and names the sheet after the file', async () => {
    const model = await parseSpreadsheet(textBytes('\uFEFFa,b\n1,2\n'), 'C:\\reports\\订单.csv');

    expect(model.sheets[0]?.name).toBe('订单');
    expect(textsOf(model.sheets[0]?.rows[1])).toEqual(['1', '2']);
  });

  it('routes payloads that are not zip archives through the CSV parser', async () => {
    const model = await parseSpreadsheet(textBytes('a,b\n'));
    expect(model.sheets[0]?.name).toBe('Sheet1');
    expect(textsOf(model.sheets[0]?.rows[0])).toEqual(['a', 'b']);
  });

  it('rejects a zip archive without a workbook part instead of decoding it as text', async () => {
    // `.ods`、Numbers、损坏的 xlsx 都是「ZIP 但没有 xl/workbook.xml」。以前这里会退回
    // 按 CSV 解析，等于把压缩容器的字节解码成一张乱码表；现在明确报错，界面给兜底面板。
    await expect(
      parseSpreadsheet(buildWorkbook({ 'docProps/app.xml': '<Properties/>' }), 'mystery.bin'),
    ).rejects.toMatchObject({ code: 'workbook-missing' });
  });

  it('keeps a sheet with an unreadable worksheet part as an empty sheet', async () => {
    const model = await parseSpreadsheet(buildWorkbook({
      'xl/workbook.xml': WORKBOOK_XML,
      'xl/worksheets/sheet1.xml': '<worksheet><sheetData><row>',
      'xl/worksheets/sheet2.xml': SHEET_TWO_XML,
    }), 'broken.xlsx');

    expect(model.sheets[0]).toEqual({
      name: '汇总',
      rows: [],
      truncatedRows: false,
      truncatedColumns: false,
    });
    expect(textsOf(model.sheets[1]?.rows[0])).toEqual(['1']);
  });
});
