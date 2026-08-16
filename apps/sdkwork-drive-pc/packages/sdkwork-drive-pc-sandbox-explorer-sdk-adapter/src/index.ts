import { parseInt as parseInteger, uuid } from '@sdkwork/utils';
import type {
  SandboxChildPage,
  SandboxCapabilities,
  SandboxEntry,
  SandboxExplorerPort,
  SandboxFileContent,
  SandboxFileEncoding,
  SandboxMutationCommand,
  SandboxPage,
  SandboxRoot,
} from '@sdkwork/drive-pc-sandbox-contracts';

const MAX_SANDBOX_PAGE_SIZE = 200;
const MAX_SANDBOX_DIRECTORY_PAGE_SIZE = 1_000;
const MAX_LOGICAL_PATH_LENGTH = 4096;
const MAX_ENTRY_NAME_LENGTH = 255;
const WINDOWS_RESERVED_ENTRY_NAME = /^(?:con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\..*)?$/iu;

export interface CreateDriveSandboxExplorerSdkPortOptions {
  readonly client: DriveSandboxExplorerSdkClient;
  readonly idempotencyKeyFactory?: () => string;
}

export interface DriveSandboxExplorerSdkPageInfo {
  readonly mode?: 'offset' | 'cursor';
  readonly page?: number;
  readonly pageSize?: number;
  readonly totalItems?: string;
  readonly totalPages?: number;
  readonly hasMore?: boolean;
  readonly nextCursor?: string | null;
}

export interface DriveSandboxExplorerSdkRoot {
  readonly id: string;
  readonly displayName: string;
  readonly rootEntryId: string;
  readonly capabilities: SandboxCapabilities;
}

export interface DriveSandboxExplorerSdkEntry {
  readonly id: string;
  readonly sandboxId: string;
  readonly parentId?: string | null;
  readonly name: string;
  readonly kind: SandboxEntry['kind'];
  readonly logicalPath: string;
  readonly revision: string;
}

export interface DriveSandboxExplorerSdkRootListData {
  readonly items: readonly DriveSandboxExplorerSdkRoot[];
  readonly pageInfo: DriveSandboxExplorerSdkPageInfo;
}

export interface DriveSandboxExplorerSdkEntryListData {
  readonly items: readonly DriveSandboxExplorerSdkEntry[];
  readonly pageInfo: DriveSandboxExplorerSdkPageInfo;
}

export interface DriveSandboxExplorerSdkFileContent {
  readonly entry: DriveSandboxExplorerSdkEntry;
  readonly encoding: SandboxFileEncoding;
  readonly content: string;
  readonly sizeBytes: string;
  readonly checksumSha256: string;
}

export interface DriveSandboxExplorerSdkMutationCommand {
  readonly accepted: true;
  readonly resourceId: string;
  readonly status: 'deleted';
}

export interface DriveSandboxExplorerSdkClient {
  readonly drive: {
    readonly sandboxes: {
      list(input: { page: number; pageSize: number }): Promise<DriveSandboxExplorerSdkRootListData>;
    };
    readonly sandboxEntries: {
      list(
        sandboxId: string,
        input: { parentPath: string; cursor?: string; pageSize: number },
      ): Promise<DriveSandboxExplorerSdkEntryListData>;
      update(
        sandboxId: string,
        entryId: string,
        body: {
          logicalPath: string;
          destinationParentPath: string;
          destinationName: string;
        },
        input: { ifMatch: string; idempotencyKey: string },
      ): Promise<DriveSandboxExplorerSdkEntry>;
      purge(
        sandboxId: string,
        entryId: string,
        body: { logicalPath: string; recursive: boolean },
        input: { ifMatch: string; idempotencyKey: string },
      ): Promise<DriveSandboxExplorerSdkMutationCommand>;
    };
    readonly sandboxDirectories: {
      create(
        sandboxId: string,
        body: { parentPath: string; name: string },
        input: { idempotencyKey: string },
      ): Promise<DriveSandboxExplorerSdkEntry>;
    };
    readonly sandboxFiles: {
      create(
        sandboxId: string,
        body: {
          parentPath: string;
          name: string;
          content: string;
          encoding: SandboxFileEncoding;
        },
        input: { idempotencyKey: string },
      ): Promise<DriveSandboxExplorerSdkEntry>;
    };
    readonly sandboxFileContents: {
      retrieve(
        sandboxId: string,
        entryId: string,
        input: { logicalPath: string; encoding?: SandboxFileEncoding },
      ): Promise<DriveSandboxExplorerSdkFileContent>;
      update(
        sandboxId: string,
        entryId: string,
        body: {
          logicalPath: string;
          content: string;
          encoding: SandboxFileEncoding;
        },
        input: { ifMatch: string; idempotencyKey: string },
      ): Promise<DriveSandboxExplorerSdkEntry>;
    };
  };
}

function assertOpaqueValue(value: string, label: string): string {
  const normalized = value.trim();
  if (!normalized || /[\u0000-\u001f\u007f]/u.test(normalized)) {
    throw new TypeError(`${label} must be a non-empty value without control characters.`);
  }
  return normalized;
}

function assertLogicalPath(value: string, allowRoot: boolean): string {
  if (value === '' && allowRoot) return value;
  if (
    !value ||
    value.length > MAX_LOGICAL_PATH_LENGTH ||
    value.startsWith('/') ||
    value.endsWith('/') ||
    value.includes('\\') ||
    /[\u0000-\u001f\u007f]/u.test(value)
  ) {
    throw new TypeError('Sandbox logical path must be a canonical relative path.');
  }
  const segments = value.split('/');
  if (segments.some((segment) => !segment || segment === '.' || segment === '..')) {
    throw new TypeError(
      'Sandbox logical path must be a canonical relative path without empty or dot segments.',
    );
  }
  return value;
}

function assertPortableEntryName(value: string, label: string): string {
  if (
    !value
    || value.length > MAX_ENTRY_NAME_LENGTH
    || value === '.'
    || value === '..'
    || value.endsWith('.')
    || value.endsWith(' ')
    || /[<>:"/\\|?*\u0000-\u001f\u007f]/u.test(value)
    || WINDOWS_RESERVED_ENTRY_NAME.test(value)
  ) {
    throw new TypeError(`${label} must be a portable single entry name.`);
  }
  return value;
}

function quoteStrongRevision(value: string): string {
  const revision = assertOpaqueValue(value, 'Sandbox revision');
  if (revision.includes('"')) {
    throw new TypeError('Sandbox revision must be the raw unquoted revision value.');
  }
  return `"${revision}"`;
}

function assertPageInput(page: number, pageSize: number, maxPageSize: number): void {
  if (!Number.isSafeInteger(page) || page < 1) {
    throw new RangeError('Sandbox page must be a positive safe integer.');
  }
  if (!Number.isSafeInteger(pageSize) || pageSize < 1 || pageSize > maxPageSize) {
    throw new RangeError(`Sandbox page size must be in range [1, ${maxPageSize}].`);
  }
}

function mapRoot(value: DriveSandboxExplorerSdkRoot): SandboxRoot {
  return {
    id: value.id,
    displayName: value.displayName,
    rootEntryId: value.rootEntryId,
    capabilities: {
      browse: value.capabilities.browse,
      createFile: value.capabilities.createFile,
      createDirectory: value.capabilities.createDirectory,
      deleteEntry: value.capabilities.deleteEntry,
      moveEntry: value.capabilities.moveEntry,
      readFile: value.capabilities.readFile,
      selectDirectory: value.capabilities.selectDirectory,
      writeFile: value.capabilities.writeFile,
    },
  };
}

function mapFileContent(value: DriveSandboxExplorerSdkFileContent): SandboxFileContent {
  return {
    entry: mapEntry(value.entry),
    encoding: value.encoding,
    content: value.content,
    sizeBytes: value.sizeBytes,
    checksumSha256: value.checksumSha256,
  };
}

function mapMutationCommand(
  value: DriveSandboxExplorerSdkMutationCommand,
): SandboxMutationCommand {
  return {
    accepted: value.accepted,
    resourceId: value.resourceId,
    status: value.status,
  };
}

function mapEntry(value: DriveSandboxExplorerSdkEntry): SandboxEntry {
  return {
    id: value.id,
    sandboxId: value.sandboxId,
    parentId: value.parentId ?? null,
    name: value.name,
    kind: value.kind,
    logicalPath: value.logicalPath,
    revision: value.revision,
  };
}

function readTotalItems(pageInfo: DriveSandboxExplorerSdkPageInfo, fallback: number): number {
  const parsed = pageInfo.totalItems ? parseInteger(pageInfo.totalItems) : null;
  return parsed !== null && parsed >= 0 ? parsed : fallback;
}

function mapSandboxPage(
  input: { readonly page: number; readonly pageSize: number },
  items: readonly DriveSandboxExplorerSdkRoot[],
  pageInfo: DriveSandboxExplorerSdkPageInfo,
): SandboxPage {
  const page = pageInfo.page ?? input.page;
  const pageSize = pageInfo.pageSize ?? input.pageSize;
  const totalItems = readTotalItems(pageInfo, (page - 1) * pageSize + items.length);
  return {
    items: items.map(mapRoot),
    page,
    pageSize,
    totalItems,
    totalPages: pageInfo.totalPages ?? Math.max(1, Math.ceil(totalItems / pageSize)),
  };
}

function mapChildPage(
  items: readonly DriveSandboxExplorerSdkEntry[],
  pageInfo: DriveSandboxExplorerSdkPageInfo,
): SandboxChildPage {
  const nextCursor = pageInfo.nextCursor ?? undefined;
  if (pageInfo.hasMore && !nextCursor) {
    throw new Error('Drive sandbox entry response is missing the required next cursor.');
  }
  return {
    items: items.map(mapEntry),
    ...(nextCursor ? { nextCursor } : {}),
  };
}

export function createDriveSandboxExplorerSdkPort(
  options: CreateDriveSandboxExplorerSdkPortOptions,
): SandboxExplorerPort {
  const { client } = options;
  const idempotencyKeyFactory = options.idempotencyKeyFactory ?? uuid;
  return {
    async listSandboxes(input) {
      assertPageInput(input.page, input.pageSize, MAX_SANDBOX_PAGE_SIZE);
      const result = await client.drive.sandboxes.list({
        page: input.page,
        pageSize: input.pageSize,
      });
      return mapSandboxPage(input, result.items, result.pageInfo);
    },
    async listChildren(input) {
      assertPageInput(1, input.pageSize, MAX_SANDBOX_DIRECTORY_PAGE_SIZE);
      const result = await client.drive.sandboxEntries.list(input.sandboxId, {
        parentPath: input.parentPath,
        pageSize: input.pageSize,
        ...(input.cursor ? { cursor: input.cursor } : {}),
      });
      return mapChildPage(result.items, result.pageInfo);
    },
    async createDirectory(input) {
      const result = await client.drive.sandboxDirectories.create(input.sandboxId, {
        parentPath: assertLogicalPath(input.parentPath, true),
        name: assertPortableEntryName(input.name, 'Sandbox directory name'),
      }, {
        idempotencyKey: idempotencyKeyFactory(),
      });
      return mapEntry(result);
    },
    async createFile(input) {
      const result = await client.drive.sandboxFiles.create(input.sandboxId, {
        parentPath: assertLogicalPath(input.parentPath, true),
        name: assertPortableEntryName(input.name, 'Sandbox file name'),
        content: input.content ?? '',
        encoding: input.encoding ?? 'utf8',
      }, {
        idempotencyKey: idempotencyKeyFactory(),
      });
      return mapEntry(result);
    },
    async readFile(input) {
      const result = await client.drive.sandboxFileContents.retrieve(
        input.sandboxId,
        input.entryId,
        {
          logicalPath: assertLogicalPath(input.logicalPath, false),
          ...(input.encoding ? { encoding: input.encoding } : {}),
        },
      );
      return mapFileContent(result);
    },
    async updateFile(input) {
      const logicalPath = assertLogicalPath(input.logicalPath, false);
      const result = await client.drive.sandboxFileContents.update(
        input.sandboxId,
        input.entryId,
        {
          logicalPath,
          content: input.content,
          encoding: input.encoding ?? 'utf8',
        },
        {
          ifMatch: quoteStrongRevision(input.revision),
          idempotencyKey: idempotencyKeyFactory(),
        },
      );
      return mapEntry(result);
    },
    async moveEntry(input) {
      const result = await client.drive.sandboxEntries.update(
        input.sandboxId,
        input.entryId,
        {
          logicalPath: assertLogicalPath(input.logicalPath, false),
          destinationParentPath: assertLogicalPath(input.destinationParentPath, true),
          destinationName: assertPortableEntryName(
            input.destinationName,
            'Sandbox destination name',
          ),
        },
        {
          ifMatch: quoteStrongRevision(input.revision),
          idempotencyKey: idempotencyKeyFactory(),
        },
      );
      return mapEntry(result);
    },
    async deleteEntry(input) {
      const result = await client.drive.sandboxEntries.purge(
        input.sandboxId,
        input.entryId,
        {
          logicalPath: assertLogicalPath(input.logicalPath, false),
          recursive: input.recursive,
        },
        {
          ifMatch: quoteStrongRevision(input.revision),
          idempotencyKey: idempotencyKeyFactory(),
        },
      );
      return mapMutationCommand(result);
    },
  };
}
