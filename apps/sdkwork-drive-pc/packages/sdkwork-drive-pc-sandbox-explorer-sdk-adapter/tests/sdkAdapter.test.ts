import { describe, expect, it, vi } from 'vitest';
import {
  createDriveSandboxExplorerSdkPort,
  type DriveSandboxExplorerSdkEntryListData,
} from '../src/index';

function createClient() {
  return {
    drive: {
      sandboxes: {
        list: vi.fn(async () => ({
          items: [{
            id: 'sandbox-1',
            displayName: 'Deployment files',
            rootEntryId: 'root-1',
            effectiveAccess: 'full' as const,
            lifecycleStatus: 'active' as const,
            capabilities: {
              browse: true,
              createFile: true,
              createDirectory: true,
              deleteEntry: true,
              moveEntry: true,
              readFile: true,
              selectDirectory: true,
              writeFile: true,
            },
            revision: '7',
          }],
          pageInfo: {
            mode: 'offset' as const,
            page: 1,
            pageSize: 50,
            totalItems: '1',
            totalPages: 1,
          },
        })),
      },
      sandboxEntries: {
        list: vi.fn(async (): Promise<DriveSandboxExplorerSdkEntryListData> => ({
          items: [{
            id: 'entry-src',
            sandboxId: 'sandbox-1',
            parentId: 'root-1',
            name: 'src',
            kind: 'directory' as const,
            logicalPath: 'projects/demo/src',
            revision: '11',
          }],
          pageInfo: {
            mode: 'cursor' as const,
            nextCursor: 'cursor-2',
            hasMore: true,
          },
        })),
        update: vi.fn(async (
          _sandboxId: string,
          entryId: string,
          body: {
            logicalPath: string;
            destinationParentPath: string;
            destinationName: string;
          },
          _input: { ifMatch: string; idempotencyKey: string },
        ) => ({
          id: entryId,
          sandboxId: 'sandbox-1',
          parentId: 'entry-archive',
          name: body.destinationName,
          kind: 'file' as const,
          logicalPath: `${body.destinationParentPath}/${body.destinationName}`,
          revision: '14',
        })),
        purge: vi.fn(async (
          _sandboxId: string,
          entryId: string,
          _body: { logicalPath: string; recursive: boolean },
          _input: { ifMatch: string; idempotencyKey: string },
        ) => ({
          accepted: true as const,
          resourceId: entryId,
          status: 'deleted' as const,
        })),
      },
      sandboxDirectories: {
        create: vi.fn(async (
          _sandboxId: string,
          body: { parentPath: string; name: string },
          _input: { idempotencyKey: string },
        ) => ({
          id: 'entry-components',
          sandboxId: 'sandbox-1',
          parentId: 'entry-src',
          name: body.name,
          kind: 'directory' as const,
          logicalPath: `${body.parentPath}/${body.name}`,
          revision: '12',
        })),
      },
      sandboxFiles: {
        create: vi.fn(async (
          _sandboxId: string,
          body: {
            parentPath: string;
            name: string;
            content: string;
            encoding: 'utf8' | 'base64';
          },
          _input: { idempotencyKey: string },
        ) => ({
          id: 'entry-readme',
          sandboxId: 'sandbox-1',
          parentId: 'entry-src',
          name: body.name,
          kind: 'file' as const,
          logicalPath: `${body.parentPath}/${body.name}`,
          revision: '12',
        })),
      },
      sandboxFileContents: {
        retrieve: vi.fn(async (
          _sandboxId: string,
          entryId: string,
          input: { logicalPath: string; encoding?: 'utf8' | 'base64' },
        ) => ({
          entry: {
            id: entryId,
            sandboxId: 'sandbox-1',
            parentId: 'entry-src',
            name: 'README.md',
            kind: 'file' as const,
            logicalPath: input.logicalPath,
            revision: '12',
          },
          encoding: input.encoding ?? 'utf8',
          content: '# Demo',
          sizeBytes: '6',
          checksumSha256: 'sha256-demo',
        })),
        update: vi.fn(async (
          _sandboxId: string,
          entryId: string,
          body: { logicalPath: string; content: string; encoding: 'utf8' | 'base64' },
          _input: { ifMatch: string; idempotencyKey: string },
        ) => ({
          id: entryId,
          sandboxId: 'sandbox-1',
          parentId: 'entry-src',
          name: 'README.md',
          kind: 'file' as const,
          logicalPath: body.logicalPath,
          revision: '13',
        })),
      },
    },
  };
}

describe('createDriveSandboxExplorerSdkPort', () => {
  it('maps paginated sandbox and logical directory SDK responses', async () => {
    const client = createClient();
    const port = createDriveSandboxExplorerSdkPort({
      client,
      idempotencyKeyFactory: () => 'directory-request-001',
    });

    await expect(port.listSandboxes({ page: 1, pageSize: 50 })).resolves.toEqual({
      items: [{
        id: 'sandbox-1',
        displayName: 'Deployment files',
        rootEntryId: 'root-1',
        capabilities: {
          browse: true,
          createFile: true,
          createDirectory: true,
          deleteEntry: true,
          moveEntry: true,
          readFile: true,
          selectDirectory: true,
          writeFile: true,
        },
      }],
      page: 1,
      pageSize: 50,
      totalItems: 1,
      totalPages: 1,
    });

    await expect(port.listChildren({
      sandboxId: 'sandbox-1',
      parentPath: 'projects/demo',
      pageSize: 1_000,
    })).resolves.toEqual({
      items: [{
        id: 'entry-src',
        sandboxId: 'sandbox-1',
        parentId: 'root-1',
        name: 'src',
        kind: 'directory',
        logicalPath: 'projects/demo/src',
        revision: '11',
      }],
      nextCursor: 'cursor-2',
    });
    expect(client.drive.sandboxEntries.list).toHaveBeenCalledWith('sandbox-1', {
      parentPath: 'projects/demo',
      pageSize: 1_000,
    });
  });

  it('maps create directory without exposing provider paths', async () => {
    const client = createClient();
    const port = createDriveSandboxExplorerSdkPort({
      client,
      idempotencyKeyFactory: () => 'directory-request-001',
    });

    await expect(port.createDirectory({
      sandboxId: 'sandbox-1',
      parentPath: 'projects/demo/src',
      name: 'components',
    })).resolves.toMatchObject({
      id: 'entry-components',
      logicalPath: 'projects/demo/src/components',
    });
    expect(client.drive.sandboxDirectories.create).toHaveBeenCalledWith(
      'sandbox-1',
      {
        parentPath: 'projects/demo/src',
        name: 'components',
      },
      { idempotencyKey: 'directory-request-001' },
    );
  });

  it('maps create, read, save, move, and delete file operations', async () => {
    const client = createClient();
    const port = createDriveSandboxExplorerSdkPort({
      client,
      idempotencyKeyFactory: () => 'mutation-request-001',
    });

    await expect(port.createFile({
      sandboxId: 'sandbox-1',
      parentPath: 'projects/demo/src',
      name: 'README.md',
      content: '# Demo',
    })).resolves.toMatchObject({
      id: 'entry-readme',
      revision: '12',
    });
    expect(client.drive.sandboxFiles.create).toHaveBeenCalledWith(
      'sandbox-1',
      {
        parentPath: 'projects/demo/src',
        name: 'README.md',
        content: '# Demo',
        encoding: 'utf8',
      },
      { idempotencyKey: 'mutation-request-001' },
    );

    await expect(port.readFile({
      sandboxId: 'sandbox-1',
      entryId: 'entry-readme',
      logicalPath: 'projects/demo/src/README.md',
    })).resolves.toMatchObject({
      content: '# Demo',
      encoding: 'utf8',
      sizeBytes: '6',
      entry: { id: 'entry-readme', revision: '12' },
    });

    await expect(port.updateFile({
      sandboxId: 'sandbox-1',
      entryId: 'entry-readme',
      logicalPath: 'projects/demo/src/README.md',
      revision: '12',
      content: '# Updated',
    })).resolves.toMatchObject({ revision: '13' });
    expect(client.drive.sandboxFileContents.update).toHaveBeenCalledWith(
      'sandbox-1',
      'entry-readme',
      {
        logicalPath: 'projects/demo/src/README.md',
        content: '# Updated',
        encoding: 'utf8',
      },
      {
        ifMatch: '"12"',
        idempotencyKey: 'mutation-request-001',
      },
    );

    await expect(port.moveEntry({
      sandboxId: 'sandbox-1',
      entryId: 'entry-readme',
      logicalPath: 'projects/demo/src/README.md',
      revision: '13',
      destinationParentPath: 'projects/demo/archive',
      destinationName: 'README.md',
    })).resolves.toMatchObject({
      logicalPath: 'projects/demo/archive/README.md',
      revision: '14',
    });
    expect(client.drive.sandboxEntries.update).toHaveBeenCalledWith(
      'sandbox-1',
      'entry-readme',
      {
        logicalPath: 'projects/demo/src/README.md',
        destinationParentPath: 'projects/demo/archive',
        destinationName: 'README.md',
      },
      {
        ifMatch: '"13"',
        idempotencyKey: 'mutation-request-001',
      },
    );

    await expect(port.deleteEntry({
      sandboxId: 'sandbox-1',
      entryId: 'entry-readme',
      logicalPath: 'projects/demo/archive/README.md',
      revision: '14',
      recursive: false,
    })).resolves.toEqual({
      accepted: true,
      resourceId: 'entry-readme',
      status: 'deleted',
    });
    expect(client.drive.sandboxEntries.purge).toHaveBeenCalledWith(
      'sandbox-1',
      'entry-readme',
      {
        logicalPath: 'projects/demo/archive/README.md',
        recursive: false,
      },
      {
        ifMatch: '"14"',
        idempotencyKey: 'mutation-request-001',
      },
    );
  });

  it('rejects non-canonical paths, unsafe names, and quoted revisions before transport', async () => {
    const client = createClient();
    const port = createDriveSandboxExplorerSdkPort({ client });

    await expect(port.createDirectory({
      sandboxId: 'sandbox-1',
      parentPath: '../outside',
      name: 'safe',
    })).rejects.toThrow(/canonical relative path/);
    await expect(port.createFile({
      sandboxId: 'sandbox-1',
      parentPath: '',
      name: '../secret',
    })).rejects.toThrow(/portable single entry name/);
    await expect(port.moveEntry({
      sandboxId: 'sandbox-1',
      entryId: 'entry-readme',
      logicalPath: 'README.md',
      revision: '"12"',
      destinationParentPath: '',
      destinationName: 'renamed.md',
    })).rejects.toThrow(/raw unquoted revision/);

    expect(client.drive.sandboxDirectories.create).not.toHaveBeenCalled();
    expect(client.drive.sandboxFiles.create).not.toHaveBeenCalled();
    expect(client.drive.sandboxEntries.update).not.toHaveBeenCalled();
  });

  it('rejects invalid pagination and incomplete cursor responses', async () => {
    const client = createClient();
    const port = createDriveSandboxExplorerSdkPort({ client });
    await expect(port.listSandboxes({ page: 0, pageSize: 50 })).rejects.toThrow(/positive/);
    await expect(port.listSandboxes({ page: 1, pageSize: 201 })).rejects.toThrow(/\[1, 200\]/);
    await expect(port.listChildren({
      sandboxId: 'sandbox-1',
      parentPath: '',
      pageSize: 1_001,
    })).rejects.toThrow(/\[1, 1000\]/);
    client.drive.sandboxEntries.list.mockResolvedValueOnce({
      items: [],
      pageInfo: { mode: 'cursor', hasMore: true },
    });
    await expect(port.listChildren({
      sandboxId: 'sandbox-1',
      parentPath: '',
      pageSize: 1_000,
    })).rejects.toThrow(/next cursor/);
  });
});
