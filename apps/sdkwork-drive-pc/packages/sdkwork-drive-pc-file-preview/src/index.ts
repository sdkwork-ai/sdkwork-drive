/**
 * sdkwork-drive-pc-file-preview —— 可复用的文件预览与编辑器组件。
 *
 * 公开面分三层，宿主按需取用：
 * - **种类与视觉**：`FilePreviewKind` 判定、`FileKindIcon` 彩色图块（列表与预览共用）；
 * - **独立预览组件**：每种格式一个组件，可脱离 Surface 单独挂在任何容器里；
 * - **预览分发**：`FilePreviewSurface` + `FilePreviewResource` 端口，宿主提供内容通道
 *   即可获得完整预览/编辑体验。
 */
export { FileKindIcon } from './components/FileKindIcon';
export type { FileKindIconProps, FileKindIconSize } from './components/FileKindIcon';
export { PreviewErrorBoundary } from './components/PreviewErrorBoundary';
export type { PreviewErrorBoundaryProps } from './components/PreviewErrorBoundary';
export { FilePreviewSurface } from './components/FilePreviewSurface';
export type { FilePreviewNavigation, FilePreviewSurfaceProps } from './components/FilePreviewSurface';
export {
  PreviewEmptyPanel,
  PreviewErrorPanel,
  PreviewLoadingPanel,
  PreviewStatePanel,
  PreviewTooLargePanel,
} from './components/PreviewStatePanels';
export type { PreviewStatePanelProps } from './components/PreviewStatePanels';
export {
  PREVIEW_BADGE_CLASS,
  PREVIEW_CANVAS_CLASS,
  PREVIEW_CHECKERBOARD_STYLE,
  PREVIEW_GHOST_BUTTON_CLASS,
  PREVIEW_ICON_BUTTON_CLASS,
  PREVIEW_INPUT_CLASS,
  PREVIEW_MONO_CLASS,
  PREVIEW_PANEL_CLASS,
  PREVIEW_PRIMARY_BUTTON_CLASS,
  PREVIEW_SCROLL_CLASS,
  PREVIEW_SECONDARY_BUTTON_CLASS,
  PREVIEW_SURFACE_CLASS,
  PREVIEW_TOOLBAR_CLASS,
} from './components/previewStyles';

export { ArchivePreview } from './components/previews/ArchivePreview';
export type { ArchivePreviewProps } from './components/previews/ArchivePreview';
export { AudioFilePreview } from './components/previews/AudioFilePreview';
export type { AudioFilePreviewProps } from './components/previews/AudioFilePreview';
export { CodeFileEditor } from './components/previews/CodeFileEditor';
export type { CodeFileEditorProps, FileEditorSaveState } from './components/previews/CodeFileEditor';
export { CodeFilePreview } from './components/previews/CodeFilePreview';
export type { CodeFilePreviewProps } from './components/previews/CodeFilePreview';
export { ImageFilePreview } from './components/previews/ImageFilePreview';
export type { ImageFilePreviewProps } from './components/previews/ImageFilePreview';
export { PdfFilePreview } from './components/previews/PdfFilePreview';
export type { PdfFilePreviewProps } from './components/previews/PdfFilePreview';
export { SlidesPreview } from './components/previews/SlidesPreview';
export type { SlidesPreviewProps } from './components/previews/SlidesPreview';
export { SpreadsheetPreview } from './components/previews/SpreadsheetPreview';
export type { SpreadsheetPreviewProps } from './components/previews/SpreadsheetPreview';
export { TextFilePreview } from './components/previews/TextFilePreview';
export type { TextFilePreviewProps } from './components/previews/TextFilePreview';
export {
  FileMetadataList,
  UnsupportedFilePreview,
} from './components/previews/UnsupportedFilePreview';
export type { UnsupportedFilePreviewProps } from './components/previews/UnsupportedFilePreview';
export { VideoFilePreview } from './components/previews/VideoFilePreview';
export type { VideoFilePreviewProps } from './components/previews/VideoFilePreview';
export { WordDocumentPreview } from './components/previews/WordDocumentPreview';
export type { WordDocumentPreviewProps } from './components/previews/WordDocumentPreview';

export {
  DEFAULT_FILE_PREVIEW_LABELS,
  formatFilePreviewLabel,
  mergeFilePreviewLabels,
} from './i18n/filePreviewLabels';
export type { FilePreviewLabels } from './i18n/filePreviewLabels';
export { useFilePreviewLabels } from './i18n/useFilePreviewLabels';
export type { FilePreviewLabelsInput } from './i18n/useFilePreviewLabels';

export { useFilePreviewContent } from './hooks/useFilePreviewContent';
export type {
  FilePreviewContentHandle,
  UseFilePreviewContentOptions,
} from './hooks/useFilePreviewContent';
export { useHostColorMode } from './hooks/useHostColorMode';
export type { HostColorMode } from './hooks/useHostColorMode';
export { useMediaPreviewUrl } from './hooks/useMediaPreviewUrl';
export type { MediaPreviewSource } from './hooks/useMediaPreviewUrl';

export {
  FILE_PREVIEW_KINDS,
  fileExtension,
  isBinaryKind,
  isDelimitedTextFile,
  isEditableKind,
  isPreviewableKind,
  isTextLikeKind,
  monacoLanguageForFile,
  prefersDirectUrl,
  resolveFilePreviewKind,
} from './kinds/filePreviewKind';
export type { FileKindInput, FilePreviewKind } from './kinds/filePreviewKind';
export { FALLBACK_FILE_KIND_VISUAL, FILE_KIND_VISUALS, fileKindVisual } from './kinds/fileKindVisuals';
export type { FileKindVisual } from './kinds/fileKindVisuals';

export {
  DEFAULT_INLINE_PREVIEW_LIMIT_BYTES,
} from './ports/filePreviewResource';
export type {
  FilePreviewContent,
  FilePreviewReadOptions,
  FilePreviewResource,
  ReadyFilePreviewContent,
} from './ports/filePreviewResource';

export {
  base64ToBytes,
  bytesToBase64,
  countTextLines,
  countTextWords,
  decodeUtf8Text,
  encodeUtf8Text,
  formatByteSize,
  looksLikeBinary,
} from './utils/bytes';
export { formatPlaybackTime } from './utils/formatPlaybackTime';
export { describePreviewError } from './utils/describePreviewError';

export { readZipArchive, isZipArchive, ZipArchiveError, MAX_ZIP_ENTRY_BYTES } from './ooxml/zipArchive';
export type { ZipArchive, ZipEntry, ZipArchiveErrorCode } from './ooxml/zipArchive';
export { WordDocumentPreviewError, parseWordDocument } from './ooxml/wordDocumentPreview';
export type {
  WordDocumentPreviewErrorCode,
  WordDocumentPreviewModel,
  WordParagraphPreview,
  WordRunPreview,
} from './ooxml/wordDocumentPreview';
export {
  MAX_SPREADSHEET_PREVIEW_COLUMNS,
  MAX_SPREADSHEET_PREVIEW_ROWS,
  SpreadsheetPreviewError,
  parseCsvSpreadsheet,
  parseSpreadsheet,
} from './ooxml/spreadsheetPreview';
export type {
  SpreadsheetCellPreview,
  SpreadsheetPreviewErrorCode,
  SpreadsheetPreviewModel,
  SpreadsheetSheetPreview,
} from './ooxml/spreadsheetPreview';
export { PresentationPreviewError, parsePresentation } from './ooxml/presentationPreview';
export type {
  PresentationPreviewErrorCode,
  PresentationPreviewModel,
  SlidePreview,
} from './ooxml/presentationPreview';
export { MAX_ARCHIVE_PREVIEW_ENTRIES, readArchivePreview } from './ooxml/archivePreview';
export type { ArchivePreviewEntry, ArchivePreviewModel } from './ooxml/archivePreview';
