import type { DriveFile } from "sdkwork-drive-pc-types";
import { getDriveFileTypeSortKey } from "../utils/fileTypeLabel";

export type FileBrowserSortField =
  | "name"
  | "owner"
  | "lastModified"
  | "contentLength"
  | "type";

export type FileBrowserSortOrder = "asc" | "desc";

export function sortDriveFiles(
  files: readonly DriveFile[],
  sortBy: FileBrowserSortField,
  sortOrder: FileBrowserSortOrder,
): DriveFile[] {
  return [...files].sort((left, right) => {
    if (left.type === "folder" && right.type !== "folder") {
      return -1;
    }
    if (left.type !== "folder" && right.type === "folder") {
      return 1;
    }

    let valueLeft: string | number;
    let valueRight: string | number;

    switch (sortBy) {
      case "name":
        valueLeft = left.name?.toLowerCase() || "";
        valueRight = right.name?.toLowerCase() || "";
        break;
      case "owner":
        valueLeft = left.ownerId?.toLowerCase() || "";
        valueRight = right.ownerId?.toLowerCase() || "";
        break;
      case "lastModified":
        valueLeft = new Date(left.updatedAt || 0).getTime();
        valueRight = new Date(right.updatedAt || 0).getTime();
        break;
      case "contentLength":
        valueLeft = left.size || 0;
        valueRight = right.size || 0;
        break;
      case "type":
        valueLeft = getDriveFileTypeSortKey(left);
        valueRight = getDriveFileTypeSortKey(right);
        break;
      default:
        return 0;
    }

    if (valueLeft < valueRight) {
      return sortOrder === "asc" ? -1 : 1;
    }
    if (valueLeft > valueRight) {
      return sortOrder === "asc" ? 1 : -1;
    }
    return 0;
  });
}
