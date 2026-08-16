import type { DriveFileService } from "sdkwork-drive-pc-core";

type SectionTitleTranslator = (key: string) => string;

type SectionTitleFileService = Pick<
  DriveFileService,
  "getKnowledgeBaseSpaces" | "getSharedSpaces"
>;

export function resolveFileBrowserSectionTitle(
  sectionKey: string,
  t: SectionTitleTranslator,
  fileService: SectionTitleFileService,
): string {
  switch (sectionKey) {
    case "my-storage":
      return t("sidebar.myStorage") || "My Storage";
    case "recent":
      return t("sidebar.recent") || "Recent Files";
    case "starred":
      return t("sidebar.starred") || "Starred Files";
    case "shared":
      return t("sidebar.sharedWithMe") || "Shared with me";
    case "computers":
      return t("sidebar.computers") || "Computers";
    case "transfer":
      return t("sidebar.transferCenter") || "Transfer Center";
    case "trash":
      return t("sidebar.trash") || "Trash";
    default: {
      const knowledgeBaseSpace = fileService
        .getKnowledgeBaseSpaces()
        .find((space) => space.id === sectionKey);
      if (knowledgeBaseSpace) {
        return knowledgeBaseSpace.name;
      }
      const remoteSpace = fileService
        .getSharedSpaces()
        .find((space) => space.id === sectionKey);
      return remoteSpace?.name || sectionKey;
    }
  }
}
