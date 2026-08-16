import type {
  DriveNode,
  DriveUploaderBlobLike,
  QuotaSummary,
  SdkworkDriveAppClient,
} from "@sdkwork/drive-app-sdk";

export type CloudDriveView = "files" | "recent";

export interface CloudFile {
  id: string;
  name: string;
  type: string;
  size: string;
  date: string;
  owner: string;
}

export interface CloudDriveStorageSummary {
  usedBytes: number;
  totalBytes?: number;
}

export interface CloudDriveRuntime {
  readonly client: SdkworkDriveAppClient;
}

export class CloudDriveCapabilityUnavailableError extends Error {
  constructor() {
    super("Cloud Drive is unavailable because the Drive owner SDK is not composed.");
    this.name = "CloudDriveCapabilityUnavailableError";
  }
}

export class CloudDrivePersonalSpaceUnavailableError extends Error {
  constructor() {
    super("Cloud Drive personal space is unavailable for the current account.");
    this.name = "CloudDrivePersonalSpaceUnavailableError";
  }
}

let runtime: CloudDriveRuntime | null = null;

function requireRuntime(): CloudDriveRuntime {
  if (!runtime) {
    throw new CloudDriveCapabilityUnavailableError();
  }
  return runtime;
}

function toFiniteNumber(value: string | undefined): number | undefined {
  if (value === undefined) {
    return undefined;
  }
  const number = Number(value);
  return Number.isFinite(number) ? number : undefined;
}

function formatFileSize(contentLength: string | undefined): string {
  const bytes = toFiniteNumber(contentLength);
  if (bytes === undefined) {
    return "-";
  }
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  if (bytes < 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

function resolveFileType(node: DriveNode): string {
  if (node.nodeType === "folder") {
    return "folder";
  }
  if (node.contentTypeGroup === "image" || node.contentTypeGroup === "video") {
    return node.contentTypeGroup;
  }
  const extension = node.fileExtension?.toLowerCase();
  if (extension === "pdf") {
    return "pdf";
  }
  if (extension && ["xls", "xlsx", "csv"].includes(extension)) {
    return "excel";
  }
  return extension || node.contentTypeGroup || "file";
}

function mapDriveNode(node: DriveNode): CloudFile {
  return {
    id: node.id,
    name: node.nodeName,
    type: resolveFileType(node),
    size: node.nodeType === "folder" ? "-" : formatFileSize(node.contentLength),
    date: new Date(node.updatedAt).toLocaleString(),
    owner: "",
  };
}

async function resolvePersonalSpaceId(client: SdkworkDriveAppClient): Promise<string> {
  const spaces = await client.drive.spaces.list({ spaceType: "personal", pageSize: 1 });
  const spaceId = spaces.items[0]?.id;
  if (!spaceId) {
    throw new CloudDrivePersonalSpaceUnavailableError();
  }
  return spaceId;
}

function mapStorageSummary(summary: QuotaSummary): CloudDriveStorageSummary {
  return {
    usedBytes: toFiniteNumber(summary.usedBytes) ?? 0,
    totalBytes: toFiniteNumber(summary.quotaBytes),
  };
}

function createShareLinkId(): string {
  const randomUuid = globalThis.crypto?.randomUUID;
  if (!randomUuid) {
    throw new Error("Secure UUID generation is unavailable in this browser.");
  }
  return randomUuid.call(globalThis.crypto);
}

export function configureCloudDriveRuntime(nextRuntime: CloudDriveRuntime): void {
  runtime = nextRuntime;
}

export function resetCloudDriveRuntime(): void {
  runtime = null;
}

export class CloudDriveService {
  static async getFiles(view: CloudDriveView = "files"): Promise<CloudFile[]> {
    const { client } = requireRuntime();
    const spaceId = await resolvePersonalSpaceId(client);
    const response = view === "recent"
      ? await client.drive.recent.list({
          spaceId,
          pageSize: "100",
          sortBy: "lastModified",
          sortOrder: "desc",
        })
      : await client.drive.nodes.list(spaceId, {
          pageSize: "100",
          sortBy: "lastModified",
          sortOrder: "desc",
        });
    return response.items.map(mapDriveNode);
  }

  static async getStorageSummary(): Promise<CloudDriveStorageSummary> {
    const { client } = requireRuntime();
    return mapStorageSummary(await client.drive.quotas.retrieve());
  }

  static async uploadFile(file: File): Promise<CloudFile> {
    const { client } = requireRuntime();
    const spaceId = await resolvePersonalSpaceId(client);
    const upload = await client.uploader.upload({
      file: file as DriveUploaderBlobLike,
      appResourceType: "mobile-file-browser",
      appResourceId: spaceId,
      scene: "drive_h5_file_upload",
      source: "h5_local_file",
      originalFileName: file.name,
      contentType: file.type || "application/octet-stream",
      spaceId,
    });
    const nodeId = upload.uploadItem.nodeId;
    if (!nodeId) {
      throw new Error("Drive upload completed without a node id.");
    }
    return mapDriveNode(await client.drive.nodes.retrieve(nodeId));
  }

  static async createFolder(name: string): Promise<CloudFile> {
    const { client } = requireRuntime();
    const spaceId = await resolvePersonalSpaceId(client);
    const node = await client.drive.nodes.folders.create({
      spaceId,
      nodeName: name,
    });
    return mapDriveNode(node);
  }

  static async deleteFile(id: string): Promise<void> {
    await requireRuntime().client.drive.nodes.delete(id);
  }

  static async renameFile(id: string, newName: string): Promise<void> {
    await requireRuntime().client.drive.nodes.update(id, { nodeName: newName });
  }

  static async createShareLink(id: string): Promise<string> {
    const response = await requireRuntime().client.drive.shareLinks.create(id, {
      id: createShareLinkId(),
      role: "reader",
    });
    if (!response.token) {
      throw new Error("Drive share link token was not returned by the server.");
    }
    return response.token;
  }

  static async claimShareLink(token: string): Promise<void> {
    await requireRuntime().client.drive.shareLinks.claim(token);
  }
}
