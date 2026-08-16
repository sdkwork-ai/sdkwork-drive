import { describe, expect, it } from 'vitest';
import {
  allocateUniqueSiblingName,
  hasSiblingNameConflict,
  resolveUniqueSiblingName,
  splitDisplayFileName,
  resolveCopyTargetName,
} from '../src/utils/resolveUniqueSiblingName';

describe('splitDisplayFileName', () => {
  it('splits stem and extension for standard files', () => {
    expect(splitDisplayFileName('report.txt')).toEqual({ stem: 'report', extension: 'txt' });
    expect(splitDisplayFileName('New folder')).toEqual({ stem: 'New folder', extension: null });
  });
});

describe('resolveUniqueSiblingName', () => {
  it('returns the base name when no sibling uses it', () => {
    expect(resolveUniqueSiblingName('New folder', ['README.md', 'Photos'])).toBe('New folder');
  });

  it('appends numbered suffix before the extension for files', () => {
    expect(resolveUniqueSiblingName('report.txt', ['report.txt'])).toBe('report (1).txt');
    expect(resolveUniqueSiblingName('report.txt', ['report.txt', 'report (1).txt'])).toBe(
      'report (2).txt',
    );
  });

  it('appends numbered suffix for extensionless names', () => {
    expect(resolveUniqueSiblingName('New folder', ['New folder'])).toBe('New folder (1)');
    expect(resolveUniqueSiblingName('New folder', ['New folder', 'New folder (1)'])).toBe(
      'New folder (2)',
    );
  });

  it('fills the lowest available suffix index', () => {
    expect(resolveUniqueSiblingName('report.txt', ['report.txt', 'report (2).txt'])).toBe(
      'report (1).txt',
    );
  });

  it('ignores the excluded name when renaming', () => {
    expect(
      resolveUniqueSiblingName('Reports', ['Reports', 'Archive'], 'Reports'),
    ).toBe('Reports');
    expect(
      hasSiblingNameConflict('Archive', ['Reports', 'Archive'], 'Reports'),
    ).toBe(true);
    expect(
      hasSiblingNameConflict('Draft', ['Reports', 'Archive'], 'Reports'),
    ).toBe(false);
  });
});

describe('resolveCopyTargetName', () => {
  it('keeps the source name when copying to an empty destination folder', () => {
    expect(resolveCopyTargetName('report.txt', [], false)).toBe('report.txt');
  });

  it('uses Copy of prefix when copying within the same parent folder', () => {
    expect(resolveCopyTargetName('report.txt', ['report.txt'], true)).toBe('Copy of report.txt');
    expect(
      resolveCopyTargetName('report.txt', ['report.txt', 'Copy of report.txt'], true),
    ).toBe('Copy of report (1).txt');
  });
});
