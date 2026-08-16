// @vitest-environment jsdom

import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { SandboxEntry, SandboxExplorerPort, SandboxRoot } from './contracts';
import { SandboxDirectoryPickerDialog } from './SandboxDirectoryPickerDialog';
import {
  SandboxDirectoryPickerProvider,
  useSandboxDirectoryPicker,
} from './SandboxDirectoryPickerProvider';
import { SandboxExplorerView } from './SandboxExplorerView';

const root: SandboxRoot = {
  id: 'sandbox-1',
  displayName: 'Server workspace',
  rootEntryId: 'root-entry-1',
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
};

const sourceDirectory: SandboxEntry = {
  id: 'entry-src',
  sandboxId: root.id,
  parentId: root.rootEntryId,
  name: 'src',
  kind: 'directory',
  logicalPath: 'src',
  revision: 'revision-src',
};

const readmeFile: SandboxEntry = {
  id: 'entry-readme',
  sandboxId: root.id,
  parentId: root.rootEntryId,
  name: 'README.md',
  kind: 'file',
  logicalPath: 'README.md',
  revision: 'revision-readme',
};

const binaryFile: SandboxEntry = {
  ...readmeFile,
  id: 'entry-binary',
  name: 'archive.zip',
  logicalPath: 'archive.zip',
  revision: 'revision-binary',
};

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
  Reflect.deleteProperty(navigator, 'clipboard');
});

class TestIntersectionObserver implements IntersectionObserver {
  static readonly instances: TestIntersectionObserver[] = [];

  readonly root: Element | Document | null;
  readonly rootMargin: string;
  readonly scrollMargin = '0px';
  readonly thresholds = [0];
  readonly observe = vi.fn();
  readonly unobserve = vi.fn();
  readonly disconnect = vi.fn();

  constructor(
    private readonly callback: IntersectionObserverCallback,
    options?: IntersectionObserverInit,
  ) {
    this.root = options?.root ?? null;
    this.rootMargin = options?.rootMargin ?? '0px';
    TestIntersectionObserver.instances.push(this);
  }

  takeRecords(): IntersectionObserverEntry[] {
    return [];
  }

  intersect(): void {
    this.callback([{ isIntersecting: true } as IntersectionObserverEntry], this);
  }
}

function explorerPort(): SandboxExplorerPort & {
  listSandboxes: ReturnType<typeof vi.fn>;
  listChildren: ReturnType<typeof vi.fn>;
  createDirectory: ReturnType<typeof vi.fn>;
} {
  return {
    listSandboxes: vi.fn(async () => ({
      items: [root],
      page: 1,
      pageSize: 50,
      totalItems: 1,
      totalPages: 1,
    })),
    listChildren: vi.fn(async ({ parentPath, cursor }) => {
      if (cursor === 'root-next') {
        return { items: [readmeFile] };
      }
      if (parentPath === 'src') {
        return { items: [] };
      }
      return { items: [sourceDirectory], nextCursor: 'root-next' };
    }),
    createDirectory: vi.fn(async ({ parentPath, name }) => ({
      id: `entry-${name}`,
      sandboxId: root.id,
      parentId: parentPath ? sourceDirectory.id : root.rootEntryId,
      name,
      kind: 'directory' as const,
      logicalPath: parentPath ? `${parentPath}/${name}` : name,
      revision: `revision-${name}`,
    })),
    createFile: vi.fn(async ({ parentPath, name }) => ({
      id: `entry-${name}`,
      sandboxId: root.id,
      parentId: parentPath ? sourceDirectory.id : root.rootEntryId,
      name,
      kind: 'file' as const,
      logicalPath: parentPath ? `${parentPath}/${name}` : name,
      revision: `revision-${name}`,
    })),
    readFile: vi.fn(async () => ({
      entry: readmeFile,
      encoding: 'utf8' as const,
      content: '',
      sizeBytes: '0',
      checksumSha256: '',
    })),
    updateFile: vi.fn(async () => readmeFile),
    moveEntry: vi.fn(async () => readmeFile),
    deleteEntry: vi.fn(async () => ({
      accepted: true as const,
      resourceId: readmeFile.id,
      status: 'deleted' as const,
    })),
  };
}

describe('SandboxExplorerView', () => {
  it('navigates into a directory without submitting the picker selection', async () => {
    const port = explorerPort();
    const onDirectorySelected = vi.fn();
    render(
      <SandboxExplorerView
        mode="select-directory"
        port={port}
        onDirectorySelected={onDirectorySelected}
      />,
    );

    await screen.findByRole('button', { name: 'src' });
    fireEvent.doubleClick(screen.getByRole('button', { name: 'src' }));

    await waitFor(() => {
      expect(port.listChildren).toHaveBeenLastCalledWith({
        sandboxId: root.id,
        parentPath: 'src',
        pageSize: 1_000,
      });
    });
    expect(onDirectorySelected).not.toHaveBeenCalled();
    expect(screen.queryByRole('button', { name: 'New folder' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'New file' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Delete' })).toBeNull();

    fireEvent.click(screen.getByRole('button', { name: 'More options' }));
    const moreMenu = screen.getByRole('menu', { name: 'More options' });
    expect(within(moreMenu).queryByRole('menuitem', { name: 'New folder' })).toBeNull();
    expect(within(moreMenu).queryByRole('menuitem', { name: 'New file' })).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'More options' }));

    fireEvent.click(screen.getByRole('button', { name: 'Select directory' }));
    expect(onDirectorySelected).toHaveBeenCalledWith({
      sandboxId: root.id,
      sandboxDisplayName: root.displayName,
      entryId: sourceDirectory.id,
      directoryName: sourceDirectory.name,
      logicalPath: 'src',
      displayPath: 'Server workspace / src',
    });
  });

  it('creates a folder below the current canonical logical path', async () => {
    const port = explorerPort();
    render(<SandboxExplorerView port={port} />);

    fireEvent.doubleClick(await screen.findByRole('button', { name: 'src' }));
    await waitFor(() => {
      expect(port.listChildren).toHaveBeenLastCalledWith({
        sandboxId: root.id,
        parentPath: 'src',
        pageSize: 1_000,
      });
    });

    fireEvent.click(screen.getByRole('button', { name: 'New folder' }));
    fireEvent.change(screen.getByLabelText('Folder name'), {
      target: { value: 'components' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Create folder' }));

    await waitFor(() => {
      expect(port.createDirectory).toHaveBeenCalledWith({
        sandboxId: root.id,
        parentPath: 'src',
        name: 'components',
      });
    });
  });

  it('focuses the address bar, shows the complete logical path, and copies it', async () => {
    const port = explorerPort();
    const writeText = vi.fn(async () => undefined);
    const clipboardDescriptor = Object.getOwnPropertyDescriptor(navigator, 'clipboard');
    Object.defineProperty(navigator, 'clipboard', {
      configurable: true,
      value: { writeText },
    });
    try {
      render(<SandboxExplorerView port={port} />);
      fireEvent.doubleClick(await screen.findByRole('button', { name: 'src' }));
      await waitFor(() => expect(port.listChildren).toHaveBeenLastCalledWith({
        sandboxId: root.id,
        parentPath: 'src',
        pageSize: 1_000,
      }));

      fireEvent.focus(screen.getByRole('navigation', { name: 'Current logical path' }));
      const addressInput = await screen.findByLabelText('Sandbox absolute path');
      expect((addressInput as HTMLInputElement).value).toBe('sandbox://sandbox-1/src');
      await waitFor(() => expect(document.activeElement).toBe(addressInput));

      fireEvent.click(screen.getByRole('button', { name: 'Copy path' }));
      await waitFor(() => expect(writeText).toHaveBeenCalledWith('sandbox://sandbox-1/src'));
      expect(screen.getByRole('button', { name: 'Path copied' })).toBeTruthy();

      fireEvent.keyDown(addressInput, { key: 'Escape' });
      expect(screen.queryByLabelText('Sandbox absolute path')).toBeNull();
      fireEvent.click(screen.getByRole('button', { name: 'src' }));
      expect(await screen.findByLabelText('Sandbox absolute path')).toBeTruthy();
      fireEvent.keyDown(screen.getByLabelText('Sandbox absolute path'), { key: 'Escape' });
      fireEvent.click(screen.getByRole('button', { name: 'Server workspace' }));
      expect(screen.queryByLabelText('Sandbox absolute path')).toBeNull();
      await waitFor(() => expect(port.listChildren).toHaveBeenLastCalledWith({
        sandboxId: root.id,
        parentPath: '',
        pageSize: 1_000,
      }));
      fireEvent.click(screen.getByRole('navigation', { name: 'Current logical path' }));
      expect(await screen.findByLabelText('Sandbox absolute path')).toBeTruthy();
    } finally {
      if (clipboardDescriptor) {
        Object.defineProperty(navigator, 'clipboard', clipboardDescriptor);
      } else {
        Reflect.deleteProperty(navigator, 'clipboard');
      }
    }
  });

  it('loads the next cursor page without replacing existing entries', async () => {
    const port = explorerPort();
    render(<SandboxExplorerView port={port} />);

    await screen.findByRole('button', { name: 'src' });
    fireEvent.click(screen.getByRole('button', { name: 'Load more' }));

    await screen.findByText('README.md');
    expect(screen.getByText('src')).toBeTruthy();
    expect(port.listChildren).toHaveBeenLastCalledWith({
      sandboxId: root.id,
      parentPath: '',
      cursor: 'root-next',
      pageSize: 1_000,
    });
  });

  it('automatically loads every cursor page as the user reaches the directory end', async () => {
    TestIntersectionObserver.instances.length = 0;
    vi.stubGlobal('IntersectionObserver', TestIntersectionObserver);
    const allEntries = Array.from({ length: 1_002 }, (_, index): SandboxEntry => ({
      id: `entry-${index}`,
      sandboxId: root.id,
      parentId: root.rootEntryId,
      name: index === 0 ? '.hidden' : index === 1_001 ? '\u8d44\u6599' : `item-${index}`,
      kind: index % 2 === 0 ? 'directory' : 'file',
      logicalPath: index === 0 ? '.hidden' : index === 1_001 ? '\u8d44\u6599' : `item-${index}`,
      revision: `revision-${index}`,
    }));
    const port = explorerPort();
    port.listChildren.mockImplementation(async ({ cursor }) => {
      if (cursor === 'page-2') return { items: allEntries.slice(1_000) };
      return { items: allEntries.slice(0, 1_000), nextCursor: 'page-2' };
    });
    render(<SandboxExplorerView port={port} />);

    await screen.findByRole('button', { name: '.hidden' });
    await waitFor(() => expect(TestIntersectionObserver.instances).toHaveLength(1));
    TestIntersectionObserver.instances[0]?.intersect();

    await screen.findByRole('button', { name: '\u8d44\u6599' });
    await waitFor(() => expect(port.listChildren).toHaveBeenCalledTimes(2));
    expect(screen.getAllByRole('button', { name: /^(?:\.hidden|item-\d+|\u8d44\u6599)$/ })).toHaveLength(1_002);
    expect(screen.getByText('1002 items loaded')).toBeTruthy();
    expect(screen.getByRole('status').textContent).toBe('All items loaded');
    expect(screen.queryByRole('button', { name: 'Load more' })).toBeNull();
  });

  it('discards a cursor page that returns after directory navigation', async () => {
    let resolveOldPage: ((page: { items: readonly SandboxEntry[] }) => void) | undefined;
    const oldPage = new Promise<{ items: readonly SandboxEntry[] }>((resolve) => {
      resolveOldPage = resolve;
    });
    const port = explorerPort();
    port.listChildren.mockImplementation(async ({ parentPath, cursor }) => {
      if (cursor === 'root-next') return oldPage;
      if (parentPath === 'src') return { items: [] };
      return { items: [sourceDirectory], nextCursor: 'root-next' };
    });
    render(<SandboxExplorerView port={port} />);

    fireEvent.click(await screen.findByRole('button', { name: 'Load more' }));
    fireEvent.doubleClick(screen.getByRole('button', { name: 'src' }));
    await waitFor(() => {
      expect(port.listChildren).toHaveBeenLastCalledWith({
        sandboxId: root.id,
        parentPath: 'src',
        pageSize: 1_000,
      });
    });

    resolveOldPage?.({ items: [readmeFile] });
    await waitFor(() => expect(screen.queryByText('README.md')).toBeNull());
    expect(screen.getByRole('button', { name: 'Server workspace' })).toBeTruthy();
  });

  it('filters the loaded page and switches between details and grid views', async () => {
    const port = explorerPort();
    const { container } = render(<SandboxExplorerView port={port} />);

    await screen.findByRole('button', { name: 'src' });
    fireEvent.click(screen.getByRole('button', { name: 'Load more' }));
    await screen.findByRole('button', { name: 'README.md' });

    fireEvent.change(screen.getByLabelText('Filter loaded items'), {
      target: { value: 'readme' },
    });
    expect(screen.queryByRole('button', { name: 'src' })).toBeNull();
    expect(screen.getByRole('button', { name: 'README.md' })).toBeTruthy();

    fireEvent.click(screen.getByRole('button', { name: 'View' }));
    expect(container.querySelector('.sdkwork-sandbox-explorer__grid-view')).toBeTruthy();
    expect(container.querySelector('.sdkwork-sandbox-explorer__details-view')).toBeNull();
  });

  it('toggles the details pane and navigates backward through directory history', async () => {
    const port = explorerPort();
    render(<SandboxExplorerView port={port} />);

    fireEvent.doubleClick(await screen.findByRole('button', { name: 'src' }));
    await waitFor(() => {
      expect(port.listChildren).toHaveBeenLastCalledWith({
        sandboxId: root.id,
        parentPath: 'src',
        pageSize: 1_000,
      });
    });

    fireEvent.click(screen.getByRole('button', { name: 'Hide details pane' }));
    expect(screen.queryByRole('complementary', { name: 'Item details' })).toBeNull();
    expect(screen.getByRole('button', { name: 'Show details pane' })).toBeTruthy();

    fireEvent.click(screen.getByRole('button', { name: 'Back' }));
    await waitFor(() => {
      expect(port.listChildren).toHaveBeenLastCalledWith({
        sandboxId: root.id,
        parentPath: '',
        pageSize: 1_000,
      });
    });
    expect(await screen.findByRole('button', { name: 'src' })).toBeTruthy();
  });

  it('exposes working keyboard shortcuts and a functional more-options menu', async () => {
    const port = explorerPort();
    render(<SandboxExplorerView port={port} />);
    await screen.findByRole('button', { name: 'src' });

    const explorer = screen.getByRole('region', { name: 'Sandbox file explorer' });
    fireEvent.keyDown(explorer, { ctrlKey: true, key: 'f' });
    expect(document.activeElement).toBe(screen.getByLabelText('Filter loaded items'));

    fireEvent.click(screen.getByRole('button', { name: 'More options' }));
    const menu = screen.getByRole('menu', { name: 'More options' });
    expect(within(menu).getByRole('menuitem', { name: /Refresh/ })).toBeTruthy();
    fireEvent.click(within(menu).getByRole('menuitem', { name: 'New folder' }));
    expect(screen.getByLabelText('Folder name')).toBeTruthy();
  });

  it('uses desktop selection semantics and opens directories only on activation', async () => {
    const port = explorerPort();
    render(<SandboxExplorerView port={port} />);
    const directoryButton = await screen.findByRole('button', { name: 'src' });
    const callsAfterLoad = port.listChildren.mock.calls.length;

    fireEvent.click(directoryButton);
    expect(port.listChildren).toHaveBeenCalledTimes(callsAfterLoad);
    expect(screen.getAllByText('File folder').length).toBeGreaterThan(0);

    fireEvent.keyDown(directoryButton, { key: 'Enter' });
    await waitFor(() => expect(port.listChildren).toHaveBeenLastCalledWith({
      sandboxId: root.id,
      parentPath: 'src',
      pageSize: 1_000,
    }));
  });

  it('uses a single tab stop with desktop arrow, boundary, and typeahead navigation', async () => {
    const alphaFile: SandboxEntry = {
      ...readmeFile,
      id: 'entry-alpha',
      name: 'alpha.txt',
      logicalPath: 'alpha.txt',
    };
    const port = explorerPort();
    port.listChildren.mockResolvedValueOnce({
      items: [readmeFile, sourceDirectory, alphaFile],
    });
    render(<SandboxExplorerView port={port} />);

    const source = await screen.findByRole('button', { name: 'src' });
    const alpha = screen.getByRole('button', { name: 'alpha.txt' });
    const readme = screen.getByRole('button', { name: 'README.md' });
    expect(source.tabIndex).toBe(0);
    expect(alpha.tabIndex).toBe(-1);
    expect(readme.tabIndex).toBe(-1);

    source.focus();
    fireEvent.keyDown(source, { key: 'ArrowDown' });
    expect(document.activeElement).toBe(alpha);
    expect(alpha.tabIndex).toBe(0);

    fireEvent.keyDown(alpha, { key: 'End' });
    expect(document.activeElement).toBe(readme);
    fireEvent.keyDown(readme, { key: 'Home' });
    expect(document.activeElement).toBe(source);
    fireEvent.keyDown(source, { key: 'r' });
    expect(document.activeElement).toBe(readme);
  });

  it('preserves desktop keyboard navigation inside a host picker dialog', async () => {
    const port = explorerPort();
    port.listChildren.mockResolvedValueOnce({
      items: [sourceDirectory, readmeFile],
    });
    render(
      <section role="dialog" aria-label="Host picker">
        <SandboxExplorerView port={port} />
      </section>,
    );

    const source = await screen.findByRole('button', { name: 'src' });
    const readme = screen.getByRole('button', { name: 'README.md' });
    source.focus();
    fireEvent.keyDown(source, { key: 'ArrowDown' });
    expect(document.activeElement).toBe(readme);
  });

  it('keeps the current directory visible during a background refresh', async () => {
    let resolveRefresh: ((page: { items: readonly SandboxEntry[] }) => void) | undefined;
    const refreshPage = new Promise<{ items: readonly SandboxEntry[] }>((resolve) => {
      resolveRefresh = resolve;
    });
    const port = explorerPort();
    port.listChildren
      .mockResolvedValueOnce({ items: [sourceDirectory] })
      .mockImplementationOnce(async () => refreshPage);
    render(<SandboxExplorerView port={port} />);

    const source = await screen.findByRole('button', { name: 'src' });
    fireEvent.click(source);
    source.focus();
    fireEvent.keyDown(source, { key: 'F5' });

    expect(screen.getByRole('button', { name: 'src' })).toBeTruthy();
    expect(await screen.findByText('Refreshing\u2026')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Refresh' }).hasAttribute('disabled')).toBe(true);

    resolveRefresh?.({ items: [sourceDirectory, readmeFile] });
    await screen.findByRole('button', { name: 'README.md' });
    expect(screen.queryByText('Refreshing\u2026')).toBeNull();
    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'src' })).toBe(document.activeElement);
    });
  });

  it('retries sandbox discovery after an initial load failure', async () => {
    const port = explorerPort();
    port.listSandboxes
      .mockRejectedValueOnce(new Error('Sandbox service unavailable.'))
      .mockResolvedValueOnce({
        items: [root],
        page: 1,
        pageSize: 50,
        totalItems: 1,
        totalPages: 1,
      });
    render(<SandboxExplorerView port={port} />);

    expect((await screen.findByRole('alert')).textContent).toContain('Sandbox service unavailable.');
    fireEvent.click(screen.getByRole('button', { name: 'Retry' }));

    await screen.findByRole('button', { name: 'src' });
    expect(port.listSandboxes).toHaveBeenCalledTimes(2);
  });

  it('creates, reads, saves, renames, and deletes files through the injected port', async () => {
    const port = explorerPort();
    render(<SandboxExplorerView port={port} />);
    await screen.findByRole('button', { name: 'src' });

    fireEvent.click(screen.getByRole('button', { name: 'New file' }));
    fireEvent.change(screen.getByLabelText('File name'), { target: { value: 'notes.txt' } });
    fireEvent.click(screen.getByRole('button', { name: 'Create file' }));
    await waitFor(() => expect(port.createFile).toHaveBeenCalledWith({
      sandboxId: root.id,
      parentPath: '',
      name: 'notes.txt',
      content: '',
      encoding: 'utf8',
    }));

    fireEvent.click(screen.getByRole('button', { name: 'Load more' }));
    const readme = await screen.findByRole('button', { name: 'README.md' });
    fireEvent.doubleClick(readme);
    expect(await screen.findByRole('dialog', { name: 'Edit README.md' })).toBeTruthy();
    await waitFor(() => expect(port.readFile).toHaveBeenCalledWith({
      sandboxId: root.id,
      entryId: readmeFile.id,
      logicalPath: readmeFile.logicalPath,
      encoding: 'utf8',
    }));
    fireEvent.change(screen.getByLabelText('File content'), { target: { value: '# Updated' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(port.updateFile).toHaveBeenCalledWith({
      sandboxId: root.id,
      entryId: readmeFile.id,
      logicalPath: readmeFile.logicalPath,
      revision: readmeFile.revision,
      content: '# Updated',
      encoding: 'utf8',
    }));
    fireEvent.click(screen.getByRole('button', { name: 'Close file' }));

    fireEvent.click(screen.getByRole('button', { name: 'README.md' }));
    fireEvent.keyDown(screen.getByRole('button', { name: 'README.md' }), { key: 'F2' });
    fireEvent.change(screen.getByLabelText('New name'), { target: { value: 'README.txt' } });
    const renameDialog = screen.getByRole('dialog', { name: 'Rename README.md' });
    fireEvent.click(within(renameDialog).getByRole('button', { name: 'Rename' }));
    await waitFor(() => expect(port.moveEntry).toHaveBeenCalled());

    fireEvent.click(screen.getByRole('button', { name: 'Load more' }));
    const refreshedReadme = await screen.findByRole('button', { name: 'README.md' });
    fireEvent.click(refreshedReadme);
    fireEvent.keyDown(refreshedReadme, { key: 'Delete' });
    expect(screen.getByRole('dialog', { name: 'Delete README.md' })).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Delete permanently' }));
    await waitFor(() => expect(port.deleteEntry).toHaveBeenCalledWith({
      sandboxId: root.id,
      entryId: readmeFile.id,
      logicalPath: readmeFile.logicalPath,
      revision: readmeFile.revision,
      recursive: false,
    }));
  });

  it('uses base64 read-only preview for binary files', async () => {
    const port = explorerPort();
    port.listChildren.mockResolvedValueOnce({ items: [binaryFile] });
    vi.mocked(port.readFile).mockResolvedValueOnce({
      entry: binaryFile,
      encoding: 'base64',
      content: 'UEsDBA==',
      sizeBytes: '4',
      checksumSha256: 'checksum',
    });
    render(<SandboxExplorerView port={port} />);

    const binaryButton = await screen.findByRole('button', { name: 'archive.zip' });
    fireEvent.doubleClick(binaryButton);
    await waitFor(() => expect(port.readFile).toHaveBeenCalledWith({
      sandboxId: root.id,
      entryId: binaryFile.id,
      logicalPath: binaryFile.logicalPath,
      encoding: 'base64',
    }));
    const content = screen.getByLabelText('File content');
    expect(content.hasAttribute('readonly')).toBe(true);
    expect(screen.queryByRole('button', { name: 'Save' })).toBeNull();
  });

  it('preserves edited content when a revision conflict rejects saving', async () => {
    const port = explorerPort();
    port.listChildren.mockResolvedValueOnce({ items: [readmeFile] });
    vi.mocked(port.readFile).mockResolvedValueOnce({
      entry: readmeFile,
      encoding: 'utf8',
      content: '# Original',
      sizeBytes: '10',
      checksumSha256: 'checksum',
    });
    vi.mocked(port.updateFile).mockRejectedValueOnce(new Error('Revision conflict'));
    render(<SandboxExplorerView port={port} />);

    fireEvent.doubleClick(await screen.findByRole('button', { name: 'README.md' }));
    const content = await screen.findByLabelText('File content');
    fireEvent.change(content, { target: { value: '# Unsaved work' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));

    expect((await screen.findByRole('alert')).textContent).toContain('Revision conflict');
    expect((screen.getByLabelText('File content') as HTMLTextAreaElement).value).toBe('# Unsaved work');
  });

  it('does not run explorer shortcuts while editing file content', async () => {
    const port = explorerPort();
    port.listChildren.mockResolvedValueOnce({ items: [readmeFile] });
    render(<SandboxExplorerView port={port} />);

    fireEvent.doubleClick(await screen.findByRole('button', { name: 'README.md' }));
    const content = await screen.findByLabelText('File content');
    fireEvent.keyDown(content, { key: 'Delete' });
    fireEvent.keyDown(content, { key: 'F2' });

    expect(screen.queryByRole('dialog', { name: 'Delete README.md' })).toBeNull();
    expect(screen.queryByRole('dialog', { name: 'Rename README.md' })).toBeNull();
  });

  it('adds an alert grid row only while a top-level error is visible', async () => {
    const port = explorerPort();
    port.listChildren.mockRejectedValueOnce(new Error('Directory unavailable'));
    render(<SandboxExplorerView port={port} />);

    const explorer = screen.getByRole('region', { name: 'Sandbox file explorer' });
    expect((await screen.findByRole('alert')).textContent).toContain('Directory unavailable');
    expect(explorer.classList.contains('has-error')).toBe(true);

    fireEvent.click(screen.getByRole('button', { name: 'Dismiss' }));
    expect(explorer.classList.contains('has-error')).toBe(false);
  });

  it('requests recursive deletion for a directory', async () => {
    const port = explorerPort();
    render(<SandboxExplorerView port={port} />);

    fireEvent.click(await screen.findByRole('button', { name: 'src' }));
    fireEvent.click(screen.getByRole('button', { name: 'Delete' }));
    fireEvent.click(screen.getByRole('button', { name: 'Delete permanently' }));

    await waitFor(() => expect(port.deleteEntry).toHaveBeenCalledWith({
      sandboxId: root.id,
      entryId: sourceDirectory.id,
      logicalPath: sourceDirectory.logicalPath,
      revision: sourceDirectory.revision,
      recursive: true,
    }));
  });

  it('moves an entry through the desktop context menu', async () => {
    const port = explorerPort();
    render(<SandboxExplorerView port={port} />);

    const source = await screen.findByRole('button', { name: 'src' });
    fireEvent.contextMenu(source, { clientX: 99999, clientY: 99999 });
    const contextMenu = screen.getByRole('menu', { name: 'src actions' });
    fireEvent.click(within(contextMenu).getByRole('menuitem', { name: /Move to/ }));
    fireEvent.change(screen.getByLabelText('Destination folder path'), {
      target: { value: 'archive' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Move' }));

    await waitFor(() => expect(port.moveEntry).toHaveBeenCalledWith({
      sandboxId: root.id,
      entryId: sourceDirectory.id,
      logicalPath: sourceDirectory.logicalPath,
      revision: sourceDirectory.revision,
      destinationParentPath: 'archive',
      destinationName: sourceDirectory.name,
    }));
  });

  it('executes copy, properties, rename, and permanent delete from the entry context menu', async () => {
    const port = explorerPort();
    const writeText = vi.fn(async () => undefined);
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText } });
    render(<SandboxExplorerView port={port} />);
    const source = await screen.findByRole('button', { name: 'src' });

    fireEvent.contextMenu(source, { clientX: 80, clientY: 80 });
    let menu = screen.getByRole('menu', { name: 'src actions' });
    fireEvent.click(within(menu).getByRole('menuitem', { name: /Copy as path/ }));
    await waitFor(() => expect(writeText).toHaveBeenCalledWith('sandbox://sandbox-1/src'));

    fireEvent.contextMenu(source, { clientX: 80, clientY: 80 });
    menu = screen.getByRole('menu', { name: 'src actions' });
    fireEvent.click(within(menu).getByRole('menuitem', { name: /Properties/ }));
    const properties = screen.getByRole('dialog', { name: 'Properties' });
    expect(within(properties).getByText('sandbox://sandbox-1/src')).toBeTruthy();
    fireEvent.click(within(properties).getByRole('button', { name: 'OK' }));

    fireEvent.contextMenu(source, { clientX: 80, clientY: 80 });
    menu = screen.getByRole('menu', { name: 'src actions' });
    fireEvent.click(within(menu).getByRole('menuitem', { name: /Rename/ }));
    expect(screen.getByRole('dialog', { name: 'Rename src' })).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));

    fireEvent.contextMenu(source, { clientX: 80, clientY: 80 });
    menu = screen.getByRole('menu', { name: 'src actions' });
    fireEvent.click(within(menu).getByRole('menuitem', { name: /Delete permanently/ }));
    expect(screen.getByRole('dialog', { name: 'Delete src' })).toBeTruthy();
  });

  it('opens directories and files from their context menus', async () => {
    const port = explorerPort();
    render(<SandboxExplorerView port={port} />);

    const source = await screen.findByRole('button', { name: 'src' });
    fireEvent.contextMenu(source, { clientX: 80, clientY: 80 });
    fireEvent.click(within(screen.getByRole('menu', { name: 'src actions' })).getByRole('menuitem', { name: /Open/ }));
    await waitFor(() => expect(port.listChildren).toHaveBeenLastCalledWith({
      sandboxId: root.id,
      parentPath: 'src',
      pageSize: 1_000,
    }));

    fireEvent.click(screen.getByRole('button', { name: 'Back' }));
    await screen.findByRole('button', { name: 'src' });
    fireEvent.click(screen.getByRole('button', { name: 'Load more' }));
    const readme = await screen.findByRole('button', { name: 'README.md' });
    fireEvent.contextMenu(readme, { clientX: 80, clientY: 80 });
    fireEvent.click(within(screen.getByRole('menu', { name: 'README.md actions' })).getByRole('menuitem', { name: /Open/ }));
    expect(await screen.findByRole('dialog', { name: 'Edit README.md' })).toBeTruthy();
    await waitFor(() => expect(port.readFile).toHaveBeenCalled());
  });

  it('provides functional current-folder context commands', async () => {
    const port = explorerPort();
    const writeText = vi.fn(async () => undefined);
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText } });
    const { container } = render(<SandboxExplorerView port={port} />);
    await screen.findByRole('button', { name: 'src' });
    const content = container.querySelector<HTMLElement>('.sdkwork-sandbox-explorer__content');
    expect(content).toBeTruthy();

    fireEvent.contextMenu(content!, { clientX: 280, clientY: 240 });
    let menu = screen.getByRole('menu', { name: 'Current folder actions' });
    fireEvent.click(within(menu).getByRole('menuitem', { name: /New folder/ }));
    expect(screen.getByLabelText('Folder name')).toBeTruthy();
    fireEvent.keyDown(screen.getByLabelText('Folder name'), { key: 'Escape' });

    fireEvent.contextMenu(content!, { clientX: 280, clientY: 240 });
    menu = screen.getByRole('menu', { name: 'Current folder actions' });
    fireEvent.click(within(menu).getByRole('menuitem', { name: /New file/ }));
    expect(screen.getByLabelText('File name')).toBeTruthy();
    fireEvent.keyDown(screen.getByLabelText('File name'), { key: 'Escape' });

    const callsBeforeRefresh = port.listChildren.mock.calls.length;
    fireEvent.contextMenu(content!, { clientX: 280, clientY: 240 });
    menu = screen.getByRole('menu', { name: 'Current folder actions' });
    fireEvent.click(within(menu).getByRole('menuitem', { name: /Refresh/ }));
    await waitFor(() => expect(port.listChildren.mock.calls.length).toBeGreaterThan(callsBeforeRefresh));

    fireEvent.contextMenu(content!, { clientX: 280, clientY: 240 });
    menu = screen.getByRole('menu', { name: 'Current folder actions' });
    fireEvent.click(within(menu).getByRole('menuitemcheckbox', { name: /Sort descending/ }));

    fireEvent.contextMenu(content!, { clientX: 280, clientY: 240 });
    menu = screen.getByRole('menu', { name: 'Current folder actions' });
    fireEvent.click(within(menu).getByRole('menuitemradio', { name: /Grid view/ }));
    expect(container.querySelector('.sdkwork-sandbox-explorer__grid-view')).toBeTruthy();

    fireEvent.contextMenu(content!, { clientX: 280, clientY: 240 });
    menu = screen.getByRole('menu', { name: 'Current folder actions' });
    fireEvent.click(within(menu).getByRole('menuitemcheckbox', { name: /Hide details pane/ }));
    expect(screen.queryByRole('complementary', { name: 'Item details' })).toBeNull();

    fireEvent.contextMenu(content!, { clientX: 280, clientY: 240 });
    menu = screen.getByRole('menu', { name: 'Current folder actions' });
    fireEvent.click(within(menu).getByRole('menuitem', { name: /Copy current path/ }));
    await waitFor(() => expect(writeText).toHaveBeenCalledWith('sandbox://sandbox-1/'));

    fireEvent.contextMenu(content!, { clientX: 280, clientY: 240 });
    menu = screen.getByRole('menu', { name: 'Current folder actions' });
    fireEvent.click(within(menu).getByRole('menuitem', { name: /Properties/ }));
    expect(screen.getByRole('dialog', { name: 'Properties' })).toBeTruthy();
  });

  it('supports keyboard context-menu navigation and restores focus on Escape', async () => {
    const port = explorerPort();
    render(<SandboxExplorerView port={port} />);
    const source = await screen.findByRole('button', { name: 'src' });
    source.focus();
    fireEvent.keyDown(source, { key: 'F10', shiftKey: true });

    const menu = screen.getByRole('menu', { name: 'src actions' });
    const openItem = within(menu).getByRole('menuitem', { name: /Open/ });
    await waitFor(() => expect(document.activeElement).toBe(openItem));
    fireEvent.keyDown(menu, { key: 'End' });
    expect(document.activeElement).toBe(within(menu).getByRole('menuitem', { name: /Delete permanently/ }));
    fireEvent.keyDown(menu, { key: 'Home' });
    expect(document.activeElement).toBe(openItem);
    fireEvent.keyDown(menu, { key: 'ArrowDown' });
    expect(document.activeElement).toBe(within(menu).getByRole('menuitem', { name: /Copy as path/ }));
    fireEvent.keyDown(menu, { key: 'Escape' });
    expect(screen.queryByRole('menu', { name: 'src actions' })).toBeNull();
    expect(document.activeElement).toBe(source);
  });

  it('keeps context menus non-mutating in directory-selection mode', async () => {
    const port = explorerPort();
    const { container } = render(<SandboxExplorerView mode="select-directory" port={port} />);
    const source = await screen.findByRole('button', { name: 'src' });
    fireEvent.contextMenu(source, { clientX: 80, clientY: 80 });
    let menu = screen.getByRole('menu', { name: 'src actions' });
    expect(within(menu).getByRole('menuitem', { name: /Open/ })).toBeTruthy();
    expect(within(menu).getByRole('menuitem', { name: /Copy as path/ })).toBeTruthy();
    expect(within(menu).getByRole('menuitem', { name: /Properties/ })).toBeTruthy();
    expect(within(menu).queryByRole('menuitem', { name: /Rename/ })).toBeNull();
    expect(within(menu).queryByRole('menuitem', { name: /Move to/ })).toBeNull();
    expect(within(menu).queryByRole('menuitem', { name: /Delete/ })).toBeNull();

    fireEvent.pointerDown(document.body);
    const content = container.querySelector<HTMLElement>('.sdkwork-sandbox-explorer__content');
    fireEvent.contextMenu(content!, { clientX: 280, clientY: 240 });
    menu = screen.getByRole('menu', { name: 'Current folder actions' });
    expect(within(menu).queryByRole('menuitem', { name: /New folder/ })).toBeNull();
    expect(within(menu).queryByRole('menuitem', { name: /New file/ })).toBeNull();
    expect(within(menu).getByRole('menuitem', { name: /Refresh/ })).toBeTruthy();
  });

  it('keeps the context menu inside every viewport edge', async () => {
    const port = explorerPort();
    render(<SandboxExplorerView port={port} />);

    const source = await screen.findByRole('button', { name: 'src' });
    fireEvent.contextMenu(source, { clientX: -20, clientY: -30 });
    const contextMenu = screen.getByRole('menu', { name: 'src actions' });
    expect(contextMenu.style.left).toBe('8px');
    expect(contextMenu.style.top).toBe('8px');
  });

  it('hides mutation controls for read-only roots', async () => {
    const readOnlyRoot: SandboxRoot = {
      ...root,
      capabilities: {
        ...root.capabilities,
        createFile: false,
        createDirectory: false,
        deleteEntry: false,
        moveEntry: false,
        writeFile: false,
      },
    };
    const port = explorerPort();
    port.listSandboxes.mockResolvedValueOnce({
      items: [readOnlyRoot],
      page: 1,
      pageSize: 50,
      totalItems: 1,
      totalPages: 1,
    });
    render(<SandboxExplorerView port={port} />);
    const entry = await screen.findByRole('button', { name: 'src' });
    fireEvent.click(entry);

    expect(screen.getByRole('button', { name: 'New folder' }).hasAttribute('disabled')).toBe(true);
    expect(screen.getByRole('button', { name: 'New file' }).hasAttribute('disabled')).toBe(true);
    expect(screen.getAllByRole('button', { name: 'Rename' }).every((button) => button.hasAttribute('disabled'))).toBe(true);
    expect(screen.getByRole('button', { name: 'Delete' }).hasAttribute('disabled')).toBe(true);

    fireEvent.contextMenu(entry, { clientX: 80, clientY: 80 });
    const menu = screen.getByRole('menu', { name: 'src actions' });
    expect(within(menu).getByRole('menuitem', { name: /Rename/ }).hasAttribute('disabled')).toBe(true);
    expect(within(menu).getByRole('menuitem', { name: /Move to/ }).hasAttribute('disabled')).toBe(true);
    expect(within(menu).getByRole('menuitem', { name: /Delete permanently/ }).hasAttribute('disabled')).toBe(true);
  });
});

describe('SandboxDirectoryPickerDialog', () => {
  it('stays unmounted while closed and closes from Escape', () => {
    const port = explorerPort();
    const onCancel = vi.fn();
    const { rerender } = render(
      <SandboxDirectoryPickerDialog
        open={false}
        port={port}
        onCancel={onCancel}
        onDirectorySelected={vi.fn()}
      />,
    );
    expect(screen.queryByRole('dialog')).toBeNull();

    rerender(
      <SandboxDirectoryPickerDialog
        open
        port={port}
        onCancel={onCancel}
        onDirectorySelected={vi.fn()}
      />,
    );
    expect(screen.getByRole('dialog')).toBeTruthy();
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Close' }));
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(onCancel).toHaveBeenCalledTimes(1);
  });

  it('maximizes and restores from window controls and header double-click', () => {
    const { container } = render(
      <SandboxDirectoryPickerDialog
        open
        port={explorerPort()}
        onCancel={vi.fn()}
        onDirectorySelected={vi.fn()}
      />,
    );
    const dialog = screen.getByRole('dialog');

    expect(dialog.classList.contains('is-maximized')).toBe(true);
    expect(screen.getByRole('button', { name: 'Restore' }).getAttribute('aria-pressed')).toBe('true');
    expect(container.querySelector('.sdkwork-sandbox-dialog-backdrop')?.classList.contains('is-maximized')).toBe(true);

    fireEvent.click(screen.getByRole('button', { name: 'Restore' }));
    expect(dialog.classList.contains('is-maximized')).toBe(false);

    fireEvent.click(screen.getByRole('button', { name: 'Maximize' }));
    expect(dialog.classList.contains('is-maximized')).toBe(true);
    expect(screen.getByRole('button', { name: 'Restore' }).getAttribute('aria-pressed')).toBe('true');
    expect(container.querySelector('.sdkwork-sandbox-dialog-backdrop')?.classList.contains('is-maximized')).toBe(true);

    const header = container.querySelector('.sdkwork-sandbox-dialog__header');
    expect(header).toBeTruthy();
    fireEvent.doubleClick(header!);
    expect(dialog.classList.contains('is-maximized')).toBe(false);
  });

  it('keeps maximized state when callback references change while open', () => {
    const { rerender } = render(
      <SandboxDirectoryPickerDialog
        open
        port={explorerPort()}
        onCancel={vi.fn()}
        onDirectorySelected={vi.fn()}
      />,
    );
    expect(screen.getByRole('button', { name: 'Restore' })).toBeTruthy();

    rerender(
      <SandboxDirectoryPickerDialog
        open
        port={explorerPort()}
        onCancel={vi.fn()}
        onDirectorySelected={vi.fn()}
      />,
    );

    expect(screen.getByRole('dialog').classList.contains('is-maximized')).toBe(true);
    expect(screen.getByRole('button', { name: 'Restore' })).toBeTruthy();
  });
});

function DirectoryPickerConsumer({
  onSelected,
}: {
  readonly onSelected: (selection: unknown) => void;
}) {
  const { pickDirectory } = useSandboxDirectoryPicker();
  return (
    <button
      type="button"
      onClick={() => void pickDirectory({ title: 'Choose project root' }).then(onSelected)}
    >
      Open picker
    </button>
  );
}

describe('SandboxDirectoryPickerProvider', () => {
  it('resolves a selected server directory and unmounts the dialog', async () => {
    const port = explorerPort();
    const onSelected = vi.fn();
    render(
      <SandboxDirectoryPickerProvider port={port}>
        <DirectoryPickerConsumer onSelected={onSelected} />
      </SandboxDirectoryPickerProvider>,
    );

    fireEvent.click(screen.getByRole('button', { name: 'Open picker' }));
    expect(await screen.findByRole('dialog', { name: 'Choose project root' })).toBeTruthy();
    await screen.findByRole('button', { name: 'src' });
    fireEvent.click(screen.getByRole('button', { name: 'Select directory' }));

    await waitFor(() => {
      expect(onSelected).toHaveBeenCalledWith({
        sandboxId: root.id,
        sandboxDisplayName: root.displayName,
        entryId: root.rootEntryId,
        directoryName: root.displayName,
        logicalPath: '',
        displayPath: 'Server workspace /',
      });
    });
    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('resolves cancellation as null', async () => {
    const onSelected = vi.fn();
    render(
      <SandboxDirectoryPickerProvider port={explorerPort()}>
        <DirectoryPickerConsumer onSelected={onSelected} />
      </SandboxDirectoryPickerProvider>,
    );

    fireEvent.click(screen.getByRole('button', { name: 'Open picker' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Close' }));

    await waitFor(() => expect(onSelected).toHaveBeenCalledWith(null));
  });
});
