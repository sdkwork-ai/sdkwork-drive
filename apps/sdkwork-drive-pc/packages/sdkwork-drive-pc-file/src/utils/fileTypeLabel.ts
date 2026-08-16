import type { DriveFile } from "sdkwork-drive-pc-types";

type FileTypeTranslate = (key: string) => string | undefined;

export function formatDriveFileTypeLabel(
  file: Pick<DriveFile, "type" | "mimeType" | "name">,
  t: FileTypeTranslate,
): string {
  if (file.type === "folder") {
    return t("fileBrowser.fileTypeFolder") || "Folder";
  }

  const mime = file.mimeType?.toLowerCase() ?? "";
  if (mime.includes("pdf")) {
    return t("fileBrowser.fileTypePdf") || "PDF";
  }
  if (mime.includes("image")) {
    return t("fileBrowser.fileTypeImage") || "Image";
  }
  if (mime.includes("video")) {
    return t("fileBrowser.fileTypeVideo") || "Video";
  }
  if (mime.includes("audio")) {
    return t("fileBrowser.fileTypeAudio") || "Audio";
  }
  if (mime.includes("zip") || mime.includes("compressed") || mime.includes("archive")) {
    return t("fileBrowser.fileTypeArchive") || "Archive";
  }
  if (mime.includes("spreadsheet") || mime.includes("csv") || mime.includes("excel")) {
    return t("fileBrowser.fileTypeSpreadsheet") || "Spreadsheet";
  }
  if (mime.includes("presentation") || mime.includes("powerpoint")) {
    return t("fileBrowser.fileTypePresentation") || "Presentation";
  }
  if (
    mime.includes("word") ||
    mime.includes("document") ||
    mime.includes("text/plain") ||
    mime.includes("markdown")
  ) {
    return t("fileBrowser.fileTypeDocument") || "Document";
  }

  const extension = extractFileExtension(file.name);
  if (extension) {
    return extension.toUpperCase();
  }

  return t("fileBrowser.fileTypeFile") || "File";
}

export function getDriveFileTypeSortKey(
  file: Pick<DriveFile, "type" | "mimeType" | "name">,
): string {
  if (file.type === "folder") {
    return "folder";
  }
  return formatDriveFileTypeLabel(file, () => undefined).toLowerCase();
}

function extractFileExtension(fileName: string): string | undefined {
  const trimmed = fileName.trim();
  const dotIndex = trimmed.lastIndexOf(".");
  if (dotIndex <= 0 || dotIndex === trimmed.length - 1) {
    return undefined;
  }
  return trimmed.slice(dotIndex + 1);
}
