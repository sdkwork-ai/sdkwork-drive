import type { FilePreviewKind } from '../kinds/filePreviewKind';
import { FILE_KIND_VISUALS } from '../kinds/fileKindVisuals';

/**
 * 预览/编辑器用到的全部文案。
 *
 * 组件不读宿主字典：宿主把本地化后的字串传进来（管理端用 drive 字典，drive 控制台
 * 用它自己的），所以同一个组件在两种语言、两个产品里都成立。缺省值只有英文，保证
 * 任何宿主漏传时界面仍然可读，而不是露出 `preview.save` 这样的 key。
 */
export interface FilePreviewLabels {
  preview: string;
  edit: string;
  editHint: string;
  /** 切换相邻文件（与文本搜索里的"上一处/下一处"是两个概念，所以命名带 File）。 */
  previousFile: string;
  nextFile: string;
  /** 位置串形如 `3 / 99`。 */
  position: string;
  save: string;
  saving: string;
  saved: string;
  saveFailed: string;
  discardChanges: string;
  unsavedChanges: string;
  unsavedCloseConfirm: string;
  cancel: string;
  download: string;
  downloadFailed: string;
  openInNewTab: string;
  close: string;
  loading: string;
  loadFailed: string;
  retry: string;
  tooLargeTitle: string;
  tooLargeHint: string;
  unavailableTitle: string;
  unavailableHint: string;
  emptyFile: string;
  zoomIn: string;
  zoomOut: string;
  rotate: string;
  fitToScreen: string;
  actualSize: string;
  wrapLines: string;
  unwrapLines: string;
  copy: string;
  copied: string;
  search: string;
  searchPlaceholder: string;
  noMatches: string;
  matchCount: string;
  previous: string;
  next: string;
  play: string;
  pause: string;
  mute: string;
  unmute: string;
  volume: string;
  playbackRate: string;
  fullscreen: string;
  sheetLabel: string;
  slideLabel: string;
  slideCount: string;
  sheetCount: string;
  unsupportedSheet: string;
  archiveEntries: string;
  archiveTruncated: string;
  archiveTotalSize: string;
  rowsTruncated: string;
  columnsTruncated: string;
  /** 解析失败但无法归因（损坏、结构不符）时的通用说明。 */
  corruptFileHint: string;
  /** 运行环境缺少解析能力（例如没有 DOMParser）。 */
  parserUnavailableHint: string;
  imageDimensions: string;
  sizeLabel: string;
  typeLabel: string;
  modifiedLabel: string;
  linesLabel: string;
  wordsLabel: string;
  charactersLabel: string;
  /** 每种文件种类的名字（列表徽标、预览头部）。 */
  kind: Record<FilePreviewKind, string>;
}

/** 英文兜底：任何宿主漏传的字段都落到这里，界面不会露出 key。 */
export const DEFAULT_FILE_PREVIEW_LABELS: FilePreviewLabels = {
  preview: 'Preview',
  edit: 'Edit',
  editHint: 'Edit this file and save it back',
  previousFile: 'Previous file',
  nextFile: 'Next file',
  position: '{index} / {total}',
  save: 'Save',
  saving: 'Saving…',
  saved: 'Saved',
  saveFailed: 'Save failed',
  discardChanges: 'Discard',
  unsavedChanges: 'Unsaved changes',
  unsavedCloseConfirm:
    'This file has unsaved changes. Closing now discards them; save first if you want to keep them.',
  cancel: 'Cancel',
  download: 'Download',
  downloadFailed: 'Download failed',
  openInNewTab: 'Open in new tab',
  close: 'Close',
  loading: 'Loading preview…',
  loadFailed: 'The preview could not be loaded',
  retry: 'Retry',
  tooLargeTitle: 'Too large to preview',
  tooLargeHint: 'This file is {size}, over the {limit} inline preview limit. Download it to open it locally.',
  unavailableTitle: 'No preview for this file',
  unavailableHint: 'This file type has no in-browser preview yet. Download it instead.',
  emptyFile: 'This file is empty',
  zoomIn: 'Zoom in',
  zoomOut: 'Zoom out',
  rotate: 'Rotate',
  fitToScreen: 'Fit to window',
  actualSize: 'Actual size',
  wrapLines: 'Wrap lines',
  unwrapLines: 'No wrap',
  copy: 'Copy',
  copied: 'Copied',
  search: 'Search',
  searchPlaceholder: 'Search in file…',
  noMatches: 'No match',
  matchCount: '{current} / {total}',
  previous: 'Previous',
  next: 'Next',
  play: 'Play',
  pause: 'Pause',
  mute: 'Mute',
  unmute: 'Unmute',
  volume: 'Volume',
  playbackRate: 'Speed',
  fullscreen: 'Fullscreen',
  sheetLabel: 'Sheet',
  slideLabel: 'Slide',
  slideCount: '{count} slides',
  sheetCount: '{count} sheets',
  unsupportedSheet: 'This sheet could not be read',
  archiveEntries: '{count} entries',
  archiveTruncated: 'Showing the first {count} entries',
  archiveTotalSize: '{size} uncompressed',
  rowsTruncated: 'Showing the first {count} rows',
  columnsTruncated: 'Showing the first {count} columns',
  corruptFileHint:
    'The file could not be read: it is damaged, or its content does not match the file type. Download it and open it locally.',
  parserUnavailableHint: 'This runtime cannot parse the file type in the browser.',
  imageDimensions: '{width} × {height}',
  sizeLabel: 'Size',
  typeLabel: 'Type',
  modifiedLabel: 'Modified',
  linesLabel: 'Lines',
  wordsLabel: 'Words',
  charactersLabel: 'Characters',
  kind: {
    folder: FILE_KIND_VISUALS.folder.defaultLabel,
    image: FILE_KIND_VISUALS.image.defaultLabel,
    video: FILE_KIND_VISUALS.video.defaultLabel,
    audio: FILE_KIND_VISUALS.audio.defaultLabel,
    pdf: FILE_KIND_VISUALS.pdf.defaultLabel,
    word: FILE_KIND_VISUALS.word.defaultLabel,
    spreadsheet: FILE_KIND_VISUALS.spreadsheet.defaultLabel,
    presentation: FILE_KIND_VISUALS.presentation.defaultLabel,
    archive: FILE_KIND_VISUALS.archive.defaultLabel,
    text: FILE_KIND_VISUALS.text.defaultLabel,
    markdown: FILE_KIND_VISUALS.markdown.defaultLabel,
    code: FILE_KIND_VISUALS.code.defaultLabel,
    unsupported: FILE_KIND_VISUALS.unsupported.defaultLabel,
  },
};

/**
 * 用宿主文案覆盖兜底文案。
 *
 * 逐字段合并（含 `kind` 子表），所以宿主可以只翻译自己关心的部分——例如只翻译动作
 * 按钮，其余继续用英文兜底；整表替换会让漏掉的字段直接显示成 `undefined`。
 */
export function mergeFilePreviewLabels(
  overrides?: Partial<FilePreviewLabels> | undefined,
): FilePreviewLabels {
  if (!overrides) {
    return DEFAULT_FILE_PREVIEW_LABELS;
  }
  return {
    ...DEFAULT_FILE_PREVIEW_LABELS,
    ...overrides,
    kind: { ...DEFAULT_FILE_PREVIEW_LABELS.kind, ...overrides.kind },
  };
}

/** `{name}` 占位符替换：预览文案里有大量计数与尺寸插值。 */
export function formatFilePreviewLabel(
  template: string,
  params?: Record<string, string | number>,
): string {
  if (!params) {
    return template;
  }
  return template.replace(/\{(\w+)\}/g, (match, key: string) =>
    Object.prototype.hasOwnProperty.call(params, key) ? String(params[key]) : match,
  );
}
