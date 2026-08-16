export type SandboxEntryKind = 'directory' | 'file';
export type SandboxFileEncoding = 'utf8' | 'base64';

export interface SandboxCapabilities {
  readonly browse: boolean;
  readonly createFile: boolean;
  readonly createDirectory: boolean;
  readonly deleteEntry: boolean;
  readonly moveEntry: boolean;
  readonly readFile: boolean;
  readonly selectDirectory: boolean;
  readonly writeFile: boolean;
}

export interface SandboxEntry {
  readonly id: string;
  readonly sandboxId: string;
  readonly parentId: string | null;
  readonly name: string;
  readonly kind: SandboxEntryKind;
  readonly logicalPath: string;
  readonly revision: string;
}

export interface SandboxRoot {
  readonly id: string;
  readonly displayName: string;
  readonly rootEntryId: string;
  readonly capabilities: SandboxCapabilities;
}

export interface SandboxChildPage {
  readonly items: readonly SandboxEntry[];
  readonly nextCursor?: string;
}

export interface SandboxPage {
  readonly items: readonly SandboxRoot[];
  readonly page: number;
  readonly pageSize: number;
  readonly totalItems: number;
  readonly totalPages: number;
}

export interface SandboxFileContent {
  readonly entry: SandboxEntry;
  readonly encoding: SandboxFileEncoding;
  readonly content: string;
  readonly sizeBytes: string;
  readonly checksumSha256: string;
}

export interface SandboxMutationCommand {
  readonly accepted: true;
  readonly resourceId: string;
  readonly status: 'deleted';
}

export interface SandboxExplorerPort {
  listSandboxes(input: { page: number; pageSize: number }): Promise<SandboxPage>;
  listChildren(input: {
    sandboxId: string;
    parentPath: string;
    cursor?: string;
    pageSize: number;
  }): Promise<SandboxChildPage>;
  createDirectory(input: {
    sandboxId: string;
    parentPath: string;
    name: string;
  }): Promise<SandboxEntry>;
  createFile(input: {
    sandboxId: string;
    parentPath: string;
    name: string;
    content?: string;
    encoding?: SandboxFileEncoding;
  }): Promise<SandboxEntry>;
  readFile(input: {
    sandboxId: string;
    entryId: string;
    logicalPath: string;
    encoding?: SandboxFileEncoding;
  }): Promise<SandboxFileContent>;
  updateFile(input: {
    sandboxId: string;
    entryId: string;
    logicalPath: string;
    revision: string;
    content: string;
    encoding?: SandboxFileEncoding;
  }): Promise<SandboxEntry>;
  moveEntry(input: {
    sandboxId: string;
    entryId: string;
    logicalPath: string;
    revision: string;
    destinationParentPath: string;
    destinationName: string;
  }): Promise<SandboxEntry>;
  deleteEntry(input: {
    sandboxId: string;
    entryId: string;
    logicalPath: string;
    revision: string;
    recursive: boolean;
  }): Promise<SandboxMutationCommand>;
}

export interface SandboxSelection {
  readonly sandboxId: string;
  readonly sandboxDisplayName: string;
  readonly entryId: string;
  readonly directoryName: string;
  readonly logicalPath: string;
  readonly displayPath: string;
}
