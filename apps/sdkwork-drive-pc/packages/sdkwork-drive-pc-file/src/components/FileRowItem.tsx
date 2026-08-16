import React, { useEffect, useRef } from "react";
import {
  Star,
  MoreHorizontal,
  Info,
  Download,
  Edit2,
  Trash,
  Trash2,
  RefreshCcw,
  Link2,
  FolderInput,
  Copy,
} from "lucide-react";
import { useTranslation } from "sdkwork-drive-pc-commons";
import type { DriveFile } from "sdkwork-drive-pc-types";
import type { DriveSection } from "../pages/DrivePage";
import { FileIcon } from "./FileIcon";
import {
  FILE_LIST_ACTIONS_CLASS,
  FILE_LIST_COL_ACTIONS_CLASS,
  FILE_LIST_ROW_CLASS,
} from "../utils/fileListLayout";
import { formatDriveFileTypeLabel } from "../utils/fileTypeLabel";

interface FileRowItemProps {
  file: DriveFile;
  activeSection: DriveSection;
  activeMenuId: string | null;
  setActiveMenuId: (id: string | null) => void;
  onToggleStar: (e: React.MouseEvent, fileId: string, fileName: string) => void;
  onDownload: (e: React.MouseEvent, file: DriveFile) => void;
  onPreview: (file: DriveFile) => void;
  onRename: (e: React.MouseEvent, file: DriveFile) => void;
  onTrashAction: (e: React.MouseEvent, file: DriveFile) => void;
  onPermanentDelete: (e: React.MouseEvent, file: DriveFile) => void;
  onShare?: (file: DriveFile) => void;
  onMove?: (file: DriveFile) => void;
  onCopy?: (file: DriveFile) => void;
  onDrillDown: (folderId: string) => void;
  formatDate: (dateString: string) => string;
  formatSize: (bytes?: number) => string;
  isInlineEditing?: boolean;
  onInlineRenameSubmit?: (newName: string) => void;
  onInlineRenameCancel?: () => void;
  isSelected?: boolean;
  onToggleSelect?: (e: React.MouseEvent, fileId: string) => void;
  hasSelection?: boolean;
  isTrashSection?: boolean;
}

export function FileRowItem({
  file,
  activeSection,
  activeMenuId,
  setActiveMenuId,
  onToggleStar,
  onDownload,
  onPreview,
  onRename,
  onTrashAction,
  onPermanentDelete,
  onShare,
  onMove,
  onCopy,
  onDrillDown,
  formatDate,
  formatSize,
  isInlineEditing = false,
  onInlineRenameSubmit,
  onInlineRenameCancel,
  isSelected = false,
  onToggleSelect,
  hasSelection = false,
  isTrashSection: isTrashSectionProp,
}: FileRowItemProps) {
  const { t } = useTranslation();
  const menuRef = useRef<HTMLDivElement>(null);

  const isItemStarred = file.isStarred;
  const isMenuOpen = activeMenuId === file.id;
  const isTrashSection = isTrashSectionProp ?? activeSection === "trash";
  const isComputerSection = activeSection === "computers";
  const hideCloudFileActions = isTrashSection || isComputerSection;

  // Handle outside clicks to close the dropdown for this item
  useEffect(() => {
    if (!isMenuOpen) return;

    const handleOutsideClick = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setActiveMenuId(null);
      }
    };

    // Timeout prevents immediate trigger during button click propagation
    const timer = setTimeout(() => {
      window.addEventListener("click", handleOutsideClick);
    }, 0);

    return () => {
      clearTimeout(timer);
      window.removeEventListener("click", handleOutsideClick);
    };
  }, [isMenuOpen, setActiveMenuId]);

  const handleRowDoubleClick = () => {
    if (file.type === "folder") {
      onDrillDown(file.id);
    } else {
      onPreview(file);
    }
  };

  const handleMenuToggle = (e: React.MouseEvent) => {
    e.stopPropagation();
    setActiveMenuId(isMenuOpen ? null : file.id);
  };

  return (
    <div
      onDoubleClick={handleRowDoubleClick}
      className={`${FILE_LIST_ROW_CLASS} group select-none ${
        isSelected ? "is-selected" : ""
      }`}
      style={{ contentVisibility: "auto", containIntrinsicSize: "0 52px" }}
    >
      {/* Checkbox Column */}
      <div className="flex items-center justify-center">
        <button
          onClick={(e) => {
            e.stopPropagation();
            onToggleSelect?.(e, file.id);
          }}
          className={`w-4 h-4 rounded border transition-all flex items-center justify-center cursor-pointer ${
            isSelected
              ? "bg-blue-600 border-blue-500 text-white opacity-100"
              : hasSelection
                ? "border-gray-300 dark:border-neutral-600 opacity-60 bg-white dark:bg-neutral-900"
                : "border-gray-300 dark:border-neutral-700 hover:border-gray-400 bg-white dark:bg-neutral-900 group-hover:opacity-100 opacity-0"
          }`}
        >
          {isSelected && (
            <svg
              className="w-2.5 h-2.5 text-white fill-none stroke-current stroke-[3]"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M5 13l4 4L19 7"
              />
            </svg>
          )}
        </button>
      </div>

      {/* Name Column */}
      <div className="sdkwork-drive-file-list-col-name group/iconcontainer">
        <div className="flex shrink-0 items-center justify-center transition-transform duration-200 group-hover/iconcontainer:scale-110">
          <FileIcon
            type={file.type}
            mimeType={file.mimeType}
            color={file.color}
          />
        </div>

        <div className="flex-1 flex items-center min-w-0">
          {isInlineEditing ? (
            <input
              type="text"
              defaultValue={file.name}
              autoFocus
              onClick={(e) => e.stopPropagation()}
              onDoubleClick={(e) => e.stopPropagation()}
              onKeyDown={(e) => {
                e.stopPropagation();
                if (e.key === "Enter") {
                  onInlineRenameSubmit?.(e.currentTarget.value);
                } else if (e.key === "Escape") {
                  onInlineRenameCancel?.();
                }
              }}
              onBlur={(e) => {
                onInlineRenameSubmit?.(e.target.value);
              }}
              className="bg-white dark:bg-[#202020] border-2 border-blue-500 dark:border-blue-500 rounded px-2 py-1 text-xs text-gray-900 dark:text-gray-100 font-medium outline-none w-full max-w-sm shrink shadow-sm focus:ring-0"
            />
          ) : (
            <div className="flex items-center gap-1.5 min-w-0 group/name">
              <span
                className="text-[13.5px] font-medium text-gray-800 dark:text-gray-200 truncate group-hover:text-blue-600 dark:group-hover:text-blue-400 transition-colors"
                title={file.name}
              >
                {file.name}
              </span>
              {!hideCloudFileActions && (
                <button
                  onClick={(e) => onToggleStar(e, file.id, file.name)}
                  className={`text-gray-300 dark:text-neutral-700 hover:text-amber-500 dark:hover:text-amber-400 transition-colors shrink-0 ${isItemStarred ? "text-amber-400 dark:text-amber-400 opacity-100" : "opacity-0 group-hover/name:opacity-100"}`}
                >
                  <Star
                    size={13}
                    className={isItemStarred ? "fill-current" : ""}
                  />
                </button>
              )}
              {!hideCloudFileActions && (
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onRename(e, file);
                  }}
                  className="p-1 opacity-0 group-hover/name:opacity-100 hover:bg-gray-100 dark:hover:bg-[#282828] text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded transition-all cursor-pointer shrink-0"
                  title={t("fileBrowser.rename")}
                >
                  <Edit2 size={12} />
                </button>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Owner Column */}
      <div className="sdkwork-drive-file-list-col-meta hidden lg:block">
        {file.ownerId}
      </div>

      {/* Size Column */}
      <div className="sdkwork-drive-file-list-col-size">
        {formatSize(file.size)}
      </div>

      {/* Type Column */}
      <div className="sdkwork-drive-file-list-col-meta">
        {formatDriveFileTypeLabel(file, t)}
      </div>

      {/* Last Modified Column */}
      <div className="sdkwork-drive-file-list-col-meta hidden font-mono lg:block">
        {formatDate(file.updatedAt)}
      </div>

      {/* Menu / Actions Button */}
      <div className={FILE_LIST_COL_ACTIONS_CLASS} ref={menuRef}>
        <div className={FILE_LIST_ACTIONS_CLASS}>
          {/* Quick Hover Download Shortcut */}
          {!hideCloudFileActions && (
            <button
              type="button"
              onClick={(e) => onDownload(e, file)}
              className="sdkwork-drive-file-list-actions__btn is-reveal"
              title={t("fileBrowser.download") || "Download"}
            >
              <Download size={15} />
            </button>
          )}

          <button
            type="button"
            onClick={handleMenuToggle}
            className={`sdkwork-drive-file-list-actions__btn is-menu ${
              isMenuOpen ? "is-active is-visible" : ""
            }`}
            title={t("fileBrowser.actionsMenu")}
          >
            <MoreHorizontal size={16} />
          </button>
        </div>

        {isMenuOpen && (
          <div className="absolute right-0 top-[calc(100%+4px)] z-50 w-48 origin-top-right animate-in fade-in zoom-in-95 rounded-xl border border-gray-100 bg-white py-1.5 text-left shadow-2xl duration-100 select-none dark:border-neutral-800 dark:bg-[#1e1e1e]">
            <button
              onClick={(e) => {
                e.stopPropagation();
                onPreview(file);
                setActiveMenuId(null);
              }}
              className="w-full flex items-center gap-2.5 px-4 py-2 text-[13px] text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-neutral-800 transition-colors cursor-pointer"
            >
              <Info size={14} className="text-gray-400 shrink-0" />{" "}
              {isComputerSection && file.type === "file"
                ? t("fileBrowser.openLocalFile") || "Open"
                : t("fileBrowser.propertiesAndInfo")}
            </button>
            {!hideCloudFileActions && (
              <>
                <button
                  onClick={(e) => {
                    onDownload(e, file);
                    setActiveMenuId(null);
                  }}
                  className="w-full flex items-center gap-2.5 px-4 py-2 text-[13px] text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-neutral-800 transition-colors cursor-pointer"
                >
                  <Download size={14} className="text-gray-400 shrink-0" />{" "}
                  {t("fileBrowser.download")}
                </button>
                <button
                  onClick={(e) => onRename(e, file)}
                  className="w-full flex items-center gap-2.5 px-4 py-2 text-[13px] text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-neutral-800 transition-colors cursor-pointer"
                >
                  <Edit2 size={14} className="text-gray-400 shrink-0" />{" "}
                  {t("fileBrowser.rename")}
                </button>

                <button
                  onClick={(e) => {
                    onToggleStar(e, file.id, file.name);
                    setActiveMenuId(null);
                  }}
                  className="w-full flex items-center gap-2.5 px-4 py-2 text-[13px] text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-neutral-800 transition-colors cursor-pointer"
                >
                  <Star
                    size={14}
                    className={`shrink-0 ${isItemStarred ? "text-amber-500 fill-amber-500" : "text-gray-400"}`}
                  />{" "}
                  {isItemStarred
                    ? t("fileBrowser.unstarResource")
                    : t("fileBrowser.starResource")}
                </button>

                {onShare ? (
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      onShare(file);
                    }}
                    className="w-full flex items-center gap-2.5 px-4 py-2 text-[13px] text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-neutral-800 transition-colors cursor-pointer"
                  >
                    <Link2 size={14} className="text-gray-400 shrink-0" />{" "}
                    {t("fileBrowser.shareLink")}
                  </button>
                ) : null}

                {onMove ? (
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      onMove(file);
                    }}
                    className="w-full flex items-center gap-2.5 px-4 py-2 text-[13px] text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-neutral-800 transition-colors cursor-pointer"
                  >
                    <FolderInput size={14} className="text-gray-400 shrink-0" />{" "}
                    {t("fileBrowser.move")}
                  </button>
                ) : null}

                {onCopy ? (
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      onCopy(file);
                    }}
                    className="w-full flex items-center gap-2.5 px-4 py-2 text-[13px] text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-neutral-800 transition-colors cursor-pointer"
                  >
                    <Copy size={14} className="text-gray-400 shrink-0" />{" "}
                    {t("fileBrowser.copy")}
                  </button>
                ) : null}

                <div className="h-px bg-gray-100 dark:bg-neutral-800 my-1 mx-2" />
              </>
            )}

            {isTrashSection ? (
              <>
                <button
                  onClick={(e) => onTrashAction(e, file)}
                  className="w-full flex items-center gap-2.5 px-4 py-2 text-[13px] text-emerald-600 dark:text-emerald-400 hover:bg-emerald-50 dark:hover:bg-emerald-950/20 transition-colors cursor-pointer"
                >
                  <RefreshCcw size={14} className="shrink-0" />{" "}
                  {t("fileBrowser.restore")}
                </button>
                <button
                  onClick={(e) => onPermanentDelete(e, file)}
                  className="w-full flex items-center gap-2.5 px-4 py-2 text-[13px] text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/20 transition-colors cursor-pointer"
                >
                  <Trash2 size={14} className="shrink-0" />{" "}
                  {t("fileBrowser.permanentDelete")}
                </button>
              </>
            ) : !isComputerSection ? (
              <button
                onClick={(e) => onTrashAction(e, file)}
                className="w-full flex items-center gap-2.5 px-4 py-2 text-[13px] text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/20 transition-colors cursor-pointer"
              >
                <Trash size={14} className="text-red-400 shrink-0" />{" "}
                {t("fileBrowser.delete")}
              </button>
            ) : null}
          </div>
        )}
      </div>
    </div>
  );
}
