import React from "react";
import {
  ArrowUpDown,
  FolderPlus,
  Grid,
  LayoutList,
  MapPin,
  Plus,
  Search,
  Trash2,
  X,
} from "lucide-react";
import { useTranslation } from "sdkwork-drive-pc-commons";
import type { DriveFile } from "sdkwork-drive-pc-types";
import { Breadcrumbs } from "./Breadcrumbs";

type SortField = "name" | "owner" | "lastModified" | "contentLength" | "type";
type SortOrder = "asc" | "desc";
type ViewMode = "list" | "grid";

export interface FileBrowserHeaderProps {
  searchQuery: string;
  onSearchQueryChange: (value: string) => void;
  sectionTitle: string;
  canCreateFolder: boolean;
  canUpload: boolean;
  canEmptyTrash?: boolean;
  onCreateFolder: () => void;
  onUpload: () => void;
  onEmptyTrash?: () => void;
  currentFolderId: string | null;
  breadcrumbFiles: DriveFile[];
  onNavigateFolder: (folderId: string | null) => void;
  itemCount: number | null;
  sortBy: SortField;
  sortOrder: SortOrder;
  onSortChange: (field: SortField, order: SortOrder) => void;
  viewMode: ViewMode;
  onViewModeChange: (mode: ViewMode) => void;
}

export function FileBrowserHeader({
  searchQuery,
  onSearchQueryChange,
  sectionTitle,
  canCreateFolder,
  canUpload,
  canEmptyTrash = false,
  onCreateFolder,
  onUpload,
  onEmptyTrash,
  currentFolderId,
  breadcrumbFiles,
  onNavigateFolder,
  itemCount,
  sortBy,
  sortOrder,
  onSortChange,
  viewMode,
  onViewModeChange,
}: FileBrowserHeaderProps) {
  const { t } = useTranslation();
  const searchPlaceholder = `${t("fileBrowser.searchPlaceholder")} ${sectionTitle}...`;

  return (
    <header className="sdkwork-drive-file-header shrink-0 select-none">
      <div className="sdkwork-drive-file-header__command">
        <div className="sdkwork-drive-file-header__search">
          <Search
            className="sdkwork-drive-file-header__search-icon"
            size={16}
            aria-hidden
          />
          <input
            type="search"
            value={searchQuery}
            onChange={(event) => onSearchQueryChange(event.target.value)}
            placeholder={searchPlaceholder}
            className="sdkwork-drive-file-header__search-input"
            aria-label={searchPlaceholder}
          />
          {searchQuery ? (
            <button
              type="button"
              onClick={() => onSearchQueryChange("")}
              className="sdkwork-drive-file-header__search-clear"
              aria-label={t("fileBrowser.cancel")}
            >
              <X size={14} />
            </button>
          ) : null}
        </div>

        {(canCreateFolder || canUpload || canEmptyTrash) && (
          <div className="sdkwork-drive-file-header__actions" role="group">
            {canEmptyTrash && onEmptyTrash && (
              <button
                type="button"
                onClick={onEmptyTrash}
                className="sdkwork-drive-file-header__btn sdkwork-drive-file-header__btn--secondary"
                title={t("fileBrowser.emptyTrash")}
              >
                <Trash2 size={15} strokeWidth={2.25} />
                <span className="hidden md:inline">{t("fileBrowser.emptyTrash")}</span>
              </button>
            )}
            {canCreateFolder && (
              <button
                type="button"
                onClick={onCreateFolder}
                className="sdkwork-drive-file-header__btn sdkwork-drive-file-header__btn--secondary"
                aria-label={t("fileBrowser.createFolder")}
                title={t("fileBrowser.createFolder")}
              >
                <FolderPlus size={15} strokeWidth={2.25} />
                <span className="hidden md:inline">{t("fileBrowser.newFolder")}</span>
              </button>
            )}
            {canUpload && (
              <button
                type="button"
                onClick={onUpload}
                className="sdkwork-drive-file-header__btn sdkwork-drive-file-header__btn--primary"
                title={t("sidebar.upload")}
              >
                <Plus size={15} strokeWidth={2.75} />
                <span className="hidden md:inline">{t("sidebar.upload")}</span>
              </button>
            )}
          </div>
        )}
      </div>

      <div className="sdkwork-drive-file-header__location">
        <div className="sdkwork-drive-file-header__path">
          <span className="sdkwork-drive-file-header__location-label hidden sm:inline-flex">
            <MapPin size={11} strokeWidth={2.5} aria-hidden />
            {t("fileBrowser.currentLocation")}
          </span>
          <div className="sdkwork-drive-file-header__breadcrumb">
            <Breadcrumbs
              variant="inline"
              currentFolderId={currentFolderId}
              allFiles={breadcrumbFiles}
              sectionTitle={sectionTitle}
              onNavigate={onNavigateFolder}
            />
          </div>
        </div>

        <div className="sdkwork-drive-file-header__meta">
          {itemCount !== null && (
            <span className="sdkwork-drive-file-header__count">
              {itemCount}{" "}
              {itemCount === 1
                ? t("fileBrowser.itemsSingular")
                : t("fileBrowser.itemsPlural")}
            </span>
          )}

          <div className="sdkwork-drive-file-header__toolbar">
            <div className="sdkwork-drive-file-header__sort">
              <select
                value={`${sortBy}-${sortOrder}`}
                onChange={(event) => {
                  const [field, order] = event.target.value.split("-") as [
                    SortField,
                    SortOrder,
                  ];
                  onSortChange(field, order);
                }}
                className="sdkwork-drive-file-header__sort-select"
                title={t("fileBrowser.sortBy")}
                aria-label={t("fileBrowser.sortBy")}
              >
                <option value="name-asc">
                  {t("fileBrowser.sortByName")} (A-Z)
                </option>
                <option value="name-desc">
                  {t("fileBrowser.sortByName")} (Z-A)
                </option>
                <option value="lastModified-desc">
                  {t("fileBrowser.sortByModified")} (Newest)
                </option>
                <option value="lastModified-asc">
                  {t("fileBrowser.sortByModified")} (Oldest)
                </option>
                <option value="contentLength-desc">
                  {t("fileBrowser.sortBySize")} (Largest)
                </option>
                <option value="contentLength-asc">
                  {t("fileBrowser.sortBySize")} (Smallest)
                </option>
                <option value="type-asc">
                  {t("fileBrowser.sortByType")} (A-Z)
                </option>
                <option value="type-desc">
                  {t("fileBrowser.sortByType")} (Z-A)
                </option>
                <option value="owner-asc">
                  {t("fileBrowser.sortByOwner")} (A-Z)
                </option>
                <option value="owner-desc">
                  {t("fileBrowser.sortByOwner")} (Z-A)
                </option>
              </select>
              <ArrowUpDown
                size={11}
                className="sdkwork-drive-file-header__sort-icon"
                aria-hidden
              />
            </div>

            <div
              className="sdkwork-drive-file-header__view-switch"
              role="group"
              aria-label={t("fileBrowser.viewMode")}
            >
              <button
                type="button"
                onClick={() => onViewModeChange("list")}
                className={`sdkwork-drive-file-header__view-btn ${
                  viewMode === "list" ? "is-active" : ""
                }`}
                title={t("fileBrowser.list")}
                aria-label={t("fileBrowser.list")}
                aria-pressed={viewMode === "list"}
              >
                <LayoutList size={14} strokeWidth={2.25} />
              </button>
              <button
                type="button"
                onClick={() => onViewModeChange("grid")}
                className={`sdkwork-drive-file-header__view-btn ${
                  viewMode === "grid" ? "is-active" : ""
                }`}
                title={t("fileBrowser.grid")}
                aria-label={t("fileBrowser.grid")}
                aria-pressed={viewMode === "grid"}
              >
                <Grid size={14} strokeWidth={2.25} />
              </button>
            </div>
          </div>
        </div>
      </div>
    </header>
  );
}
