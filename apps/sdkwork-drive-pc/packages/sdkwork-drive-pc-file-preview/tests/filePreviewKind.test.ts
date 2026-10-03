import { describe, expect, it } from 'vitest';
import {
  fileExtension,
  isEditableKind,
  isPreviewableKind,
  monacoLanguageForFile,
  resolveFilePreviewKind,
} from '../src/kinds/filePreviewKind';

describe('resolveFilePreviewKind', () => {
  it('classifies by extension before content type', () => {
    // 上传方常把 content_type 写成 octet-stream；扩展名才是用户意图。
    expect(resolveFilePreviewKind({ name: 'a/b.png', contentType: 'application/octet-stream' })).toBe('image');
    expect(resolveFilePreviewKind({ name: 'a/report.docx' })).toBe('word');
    expect(resolveFilePreviewKind({ name: 'a/budget.xlsx' })).toBe('spreadsheet');
    expect(resolveFilePreviewKind({ name: 'a/deck.pptx' })).toBe('presentation');
    expect(resolveFilePreviewKind({ name: 'a/bundle.zip' })).toBe('archive');
    expect(resolveFilePreviewKind({ name: 'a/main.ts' })).toBe('code');
    expect(resolveFilePreviewKind({ name: 'a/readme.md' })).toBe('markdown');
    expect(resolveFilePreviewKind({ name: 'a/notes.txt' })).toBe('text');
  });

  it('falls back to the content type when there is no usable extension', () => {
    expect(resolveFilePreviewKind({ name: 'export', contentType: 'image/webp' })).toBe('image');
    expect(resolveFilePreviewKind({ name: 'export', contentType: 'application/pdf' })).toBe('pdf');
    expect(
      resolveFilePreviewKind({
        name: 'export',
        contentType:
          'application/vnd.openxmlformats-officedocument.presentationml.presentation',
      }),
    ).toBe('presentation');
  });

  it('recognizes well-known extensionless files by name', () => {
    expect(resolveFilePreviewKind({ name: 'Dockerfile' })).toBe('code');
    expect(resolveFilePreviewKind({ name: 'LICENSE' })).toBe('text');
  });

  it('treats folders and trailing slashes as folders', () => {
    expect(resolveFilePreviewKind({ name: 'photos/', isFolder: true })).toBe('folder');
    expect(resolveFilePreviewKind({ name: 'photos/' })).toBe('folder');
  });

  it('answers unsupported for unknown binaries', () => {
    expect(resolveFilePreviewKind({ name: 'a/blob.bin' })).toBe('unsupported');
    expect(resolveFilePreviewKind({ name: 'a/blob', contentType: 'application/octet-stream' })).toBe(
      'unsupported',
    );
  });
});

describe('fileExtension', () => {
  it('reads the last segment extension in lower case', () => {
    expect(fileExtension('a/b/Report.PDF')).toBe('pdf');
    expect(fileExtension('archive.tar.gz')).toBe('gz');
  });

  it('answers empty for extensionless names and dotfiles', () => {
    expect(fileExtension('Dockerfile')).toBe('');
    expect(fileExtension('a/')).toBe('');
    expect(fileExtension('.gitignore')).toBe('');
  });
});

describe('monacoLanguageForFile', () => {
  it('maps the common languages to their VS Code ids', () => {
    expect(monacoLanguageForFile('a.ts')).toBe('typescript');
    expect(monacoLanguageForFile('a.tsx')).toBe('typescript');
    expect(monacoLanguageForFile('a.py')).toBe('python');
    expect(monacoLanguageForFile('a.yml')).toBe('yaml');
    expect(monacoLanguageForFile('a.md')).toBe('markdown');
    expect(monacoLanguageForFile('Dockerfile')).toBe('dockerfile');
  });

  it('falls back to plaintext for unknown extensions', () => {
    expect(monacoLanguageForFile('a.unknownext')).toBe('plaintext');
    expect(monacoLanguageForFile('a')).toBe('plaintext');
  });
});

describe('kind capabilities', () => {
  it('separates previewable, editable and binary kinds', () => {
    expect(isPreviewableKind('image')).toBe(true);
    expect(isPreviewableKind('folder')).toBe(false);
    expect(isPreviewableKind('unsupported')).toBe(false);
    expect(isEditableKind('code')).toBe(true);
    expect(isEditableKind('text')).toBe(true);
    expect(isEditableKind('image')).toBe(false);
  });
});
