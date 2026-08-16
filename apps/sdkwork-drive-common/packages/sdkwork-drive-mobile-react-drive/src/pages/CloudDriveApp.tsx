import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Filter, HardDrive, Plus, Search } from "lucide-react";
import { motion } from "motion/react";
import { useTranslation } from "react-i18next";

import {
  ActionSheet,
  IconButton,
  PageLayout,
  showToast,
} from "@sdkwork/ui-mobile-react";

import { CloudDriveActionGrid } from "../components/CloudDriveActionGrid";
import { CloudDriveFileItem } from "../components/CloudDriveFileItem";
import { CloudDriveHeaderStats } from "../components/CloudDriveHeaderStats";
import {
  CloudDriveService,
  type CloudDriveStorageSummary,
  type CloudDriveView,
  type CloudFile,
} from "../services/CloudDriveService";

type CloudDriveFilter = "all" | "files" | "folders";

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function CloudDriveApp() {
  const { t } = useTranslation("drive");
  const [activeTab, setActiveTab] = useState<CloudDriveView>("files");
  const [files, setFiles] = useState<CloudFile[]>([]);
  const [storageSummary, setStorageSummary] = useState<CloudDriveStorageSummary | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isFabSheetOpen, setIsFabSheetOpen] = useState(false);
  const [isFilterSheetOpen, setIsFilterSheetOpen] = useState(false);
  const [activeFile, setActiveFile] = useState<string | null>(null);
  const [filter, setFilter] = useState<CloudDriveFilter>("all");
  const [searchQuery, setSearchQuery] = useState("");
  const fileInputRef = useRef<HTMLInputElement>(null);

  const loadFiles = useCallback(async (view: CloudDriveView) => {
    setIsLoading(true);
    try {
      setFiles(await CloudDriveService.getFiles(view));
    } catch (error: unknown) {
      setFiles([]);
      showToast(errorMessage(error));
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadFiles(activeTab);
  }, [activeTab, loadFiles]);

  useEffect(() => {
    void CloudDriveService.getStorageSummary()
      .then(setStorageSummary)
      .catch(() => setStorageSummary(null));
  }, []);

  const visibleFiles = useMemo(() => {
    const normalizedQuery = searchQuery.trim().toLocaleLowerCase();
    return files.filter((file) => {
      if (filter === "folders" && file.type !== "folder") {
        return false;
      }
      if (filter === "files" && file.type === "folder") {
        return false;
      }
      return !normalizedQuery || file.name.toLocaleLowerCase().includes(normalizedQuery);
    });
  }, [files, filter, searchQuery]);

  const runMutation = useCallback(async (
    mutation: () => Promise<void>,
    successMessage?: string,
  ) => {
    try {
      await mutation();
      await loadFiles(activeTab);
      if (successMessage) {
        showToast(successMessage);
      }
    } catch (error: unknown) {
      showToast(errorMessage(error));
    }
  }, [activeTab, loadFiles]);

  const chooseUploadFile = () => {
    fileInputRef.current?.click();
  };

  const uploadSelectedFile = async (file: File | undefined) => {
    if (!file) {
      return;
    }
    await runMutation(
      async () => {
        await CloudDriveService.uploadFile(file);
      },
      t("file_uploaded"),
    );
  };

  const promptForSearch = () => {
    const value = window.prompt(t("search_prompt"), searchQuery);
    if (value !== null) {
      setSearchQuery(value.trim());
    }
  };

  return (
    <PageLayout title={t("title")}>
      <div className="flex flex-col h-full bg-[#f5f6f8] dark:bg-[#1a1b1c]">
        <CloudDriveHeaderStats summary={storageSummary} />
        <CloudDriveActionGrid
          activeTab={activeTab}
          setActiveTab={setActiveTab}
          onUnavailableAction={() => showToast(t("action_unavailable"))}
        />

        <div className="flex-1 overflow-y-auto px-4 pb-20">
          <div className="flex items-center justify-between py-3 px-1">
            <h2 className="text-[14px] font-medium text-text-sub">
              {activeTab === "recent" ? t("sections.recent_used") : t("sections.all_files")}
            </h2>
            <div className="flex gap-2">
              <IconButton
                icon={<Filter className="w-4 h-4 text-text-sub" />}
                className="bg-white dark:bg-[#2c2d2e] p-1.5 w-auto h-auto rounded-md shadow-sm"
                onClick={() => setIsFilterSheetOpen(true)}
              />
              <IconButton
                icon={<Search className="w-4 h-4 text-text-sub" />}
                className="bg-white dark:bg-[#2c2d2e] p-1.5 w-auto h-auto rounded-md shadow-sm"
                onClick={promptForSearch}
              />
            </div>
          </div>

          <div className="flex flex-col gap-2">
            {isLoading ? (
              <div className="flex flex-col items-center justify-center py-20 text-text-sub opacity-70">
                <div className="w-8 h-8 rounded-full border-4 border-text-sub border-t-transparent animate-spin mb-3" />
                <p className="text-[14px]">{t("loading")}</p>
              </div>
            ) : visibleFiles.length > 0 ? (
              visibleFiles.map((file) => (
                <CloudDriveFileItem
                  key={file.id}
                  file={file}
                  setActiveFile={setActiveFile}
                />
              ))
            ) : (
              <div className="flex flex-col items-center justify-center py-20 text-text-sub opacity-70">
                <HardDrive className="w-12 h-12 mb-3 stroke-current opacity-40" />
                <span className="text-[14px]">{t("no_files")}</span>
              </div>
            )}
          </div>
        </div>

        <input
          ref={fileInputRef}
          type="file"
          className="hidden"
          onChange={(event) => {
            void uploadSelectedFile(event.target.files?.[0]);
            event.target.value = "";
          }}
        />
        <motion.button
          type="button"
          aria-label={t("upload_file_title")}
          whileTap={{ scale: 0.9 }}
          whileHover={{ scale: 1.05 }}
          onClick={() => setIsFabSheetOpen(true)}
          className="absolute bottom-6 right-6 w-14 h-14 bg-gradient-to-tr from-blue-600 to-primary-blue text-white rounded-full flex items-center justify-center shadow-lg shadow-blue-500/30 z-10"
        >
          <Plus className="w-7 h-7" />
        </motion.button>
      </div>

      <ActionSheet
        isOpen={isFabSheetOpen}
        onClose={() => setIsFabSheetOpen(false)}
        title={t("upload_file_title")}
        options={[
          {
            label: t("new_folder"),
            onClick: () => {
              const name = window.prompt(t("enter_folder_name"), t("default_folder_name"));
              if (name?.trim()) {
                void runMutation(
                  async () => {
                    await CloudDriveService.createFolder(name.trim());
                  },
                  t("folder_created"),
                );
              }
            },
          },
          { label: t("upload_file"), onClick: chooseUploadFile },
        ]}
      />

      <ActionSheet
        isOpen={isFilterSheetOpen}
        onClose={() => setIsFilterSheetOpen(false)}
        title={t("filter_title")}
        options={[
          { label: t("filter_all"), onClick: () => setFilter("all") },
          { label: t("filter_files"), onClick: () => setFilter("files") },
          { label: t("filter_folders"), onClick: () => setFilter("folders") },
        ]}
      />

      <ActionSheet
        isOpen={activeFile !== null}
        onClose={() => setActiveFile(null)}
        title={t("file_actions_title")}
        options={[
          {
            label: t("share"),
            onClick: () => {
              if (!activeFile) {
                return;
              }
              void runMutation(async () => {
                const token = await CloudDriveService.createShareLink(activeFile);
                const shareUrl = `${window.location.origin}${window.location.pathname}#/workspace/drive/share/${encodeURIComponent(token)}`;
                await navigator.clipboard.writeText(shareUrl);
              }, t("link_copied"));
            },
          },
          {
            label: t("rename"),
            onClick: () => {
              const file = files.find((candidate) => candidate.id === activeFile);
              if (!activeFile || !file) {
                return;
              }
              const newName = window.prompt(t("enter_new_name"), file.name);
              if (newName?.trim()) {
                void runMutation(async () => {
                  await CloudDriveService.renameFile(activeFile, newName.trim());
                });
              }
            },
          },
          {
            label: t("delete"),
            danger: true,
            onClick: () => {
              if (activeFile) {
                void runMutation(async () => {
                  await CloudDriveService.deleteFile(activeFile);
                }, t("file_deleted"));
              }
            },
          },
        ]}
      />
    </PageLayout>
  );
}
