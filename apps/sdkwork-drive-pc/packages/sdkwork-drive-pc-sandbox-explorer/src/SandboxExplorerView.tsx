/// <reference path="./styles.d.ts" />

import {
  ArrowLeft,
  ArrowDown,
  ArrowRight,
  ArrowUp,
  Check,
  ChevronDown,
  ChevronRight,
  Copy,
  File,
  FilePenLine,
  FilePlus2,
  Folder,
  FolderOpen,
  FolderPlus,
  HardDrive,
  Info,
  LayoutGrid,
  List,
  LoaderCircle,
  MoreHorizontal,
  Move,
  PanelRight,
  Pencil,
  RefreshCw,
  Save,
  Search,
  Server,
  Trash2,
  Upload,
  X,
} from 'lucide-react';
import {
  type FormEvent,
  useCallback,
  useDeferredValue,
  useEffect,
  useId,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from 'react';
import type {
  SandboxEntry,
  SandboxExplorerPort,
  SandboxRoot,
  SandboxSelection,
} from './contracts';
import { requireDriveSandboxExplorerPort } from './runtime';
import './SandboxExplorerView.css';

const SANDBOX_PAGE_SIZE = 50;
const DIRECTORY_PAGE_SIZE = 1_000;
/** 后端 sandbox 文件内容解码后的硬限制（与 CreateDriveSandboxFileRequest 契约一致）。 */
const SANDBOX_UPLOAD_MAX_BYTES = 4 * 1024 * 1024;
const DIRECTORY_NAME_COLLATOR = new Intl.Collator(undefined, {
  numeric: true,
  sensitivity: 'base',
});
const TYPEAHEAD_RESET_DELAY_MS = 700;

interface DirectoryLocation {
  readonly entryId: string;
  readonly logicalPath: string;
}

interface BreadcrumbItem extends DirectoryLocation {
  readonly label: string;
}

interface HistoryLocation {
  readonly rootId: string;
  readonly directory: DirectoryLocation;
}

type HistoryMode = 'push' | 'replace' | 'preserve';
type ViewMode = 'details' | 'grid';

interface FileEditorState {
  readonly entry: SandboxEntry;
  readonly content: string;
  readonly encoding: 'utf8' | 'base64';
  readonly sizeBytes: string;
  readonly checksumSha256: string;
  readonly loading: boolean;
  readonly saving: boolean;
  readonly error: string | null;
}

interface EntryContextMenu {
  readonly kind: 'entry' | 'background';
  readonly entry?: SandboxEntry;
  readonly x: number;
  readonly y: number;
  readonly focusMenu: boolean;
  readonly returnFocus: HTMLElement | null;
}

interface PropertiesTarget {
  readonly kind: 'entry' | 'directory';
  readonly entry?: SandboxEntry;
}

type DesktopPlatform = 'windows' | 'macos' | 'linux';

function resolveDesktopPlatform(): DesktopPlatform {
  if (typeof navigator === 'undefined') return 'windows';
  const platform = `${navigator.userAgent} ${navigator.platform}`.toLocaleLowerCase();
  if (platform.includes('mac')) return 'macos';
  if (platform.includes('linux')) return 'linux';
  return 'windows';
}

export interface SandboxExplorerLabels {
  /** 可访问名称与可见文本；所有键均可选，缺省时使用内置英文文案。 */
  sandboxFileExplorer?: string;
  back?: string;
  forward?: string;
  parentDirectory?: string;
  refresh?: string;
  navigationHistory?: string;
  currentLogicalPath?: string;
  sandboxAbsolutePath?: string;
  noSandboxPath?: string;
  copyPath?: string;
  pathCopied?: string;
  copyFullPath?: string;
  pathCopiedAnnouncement?: string;
  filterLoadedItems?: string;
  searchPlaceholder?: string;
  clearSearch?: string;
  newFolder?: string;
  newFile?: string;
  uploadFile?: string;
  uploadingFiles?: string;
  fileTooLarge?: string;
  uploadError?: string;
  folderName?: string;
  fileName?: string;
  createFolder?: string;
  createFile?: string;
  cancelFileCreation?: string;
  rename?: string;
  delete?: string;
  sort?: string;
  sortAscending?: string;
  sortDescending?: string;
  switchToGridView?: string;
  switchToDetailsView?: string;
  view?: string;
  moreOptions?: string;
  gridView?: string;
  detailsView?: string;
  details?: string;
  hideDetailsPane?: string;
  showDetailsPane?: string;
  reload?: string;
  retry?: string;
  dismiss?: string;
  sandboxNavigation?: string;
  sandboxes?: string;
  sandbox?: string;
  availableSandboxes?: string;
  openSandbox?: string;
  moreSandboxes?: string;
  loading?: string;
  loadingFolder?: string;
  loadingFile?: string;
  noAccessibleSandboxes?: string;
  folderEmpty?: string;
  noSearchMatches?: string;
  directoryItems?: string;
  itemDetails?: string;
  nameColumn?: string;
  typeColumn?: string;
  locationColumn?: string;
  loadMore?: string;
  loadMoreItems?: string;
  loadingMoreItems?: string;
  moreItemsAvailable?: string;
  allItemsLoaded?: string;
  itemsLoadedOne?: string;
  itemsLoadedMany?: string;
  filteredFrom?: string;
  filtering?: string;
  refreshing?: string;
  noSandboxSelected?: string;
  selectDirectory?: string;
  entryActions?: string;
  currentFolderActions?: string;
  open?: string;
  openFile?: string;
  copyAsPath?: string;
  copyPathname?: string;
  copyLocation?: string;
  copyCurrentPath?: string;
  moveTo?: string;
  getInfo?: string;
  properties?: string;
  info?: string;
  deletePermanently?: string;
  selectItemDetails?: string;
  editFile?: string;
  closeFile?: string;
  fileContent?: string;
  binaryPreview?: string;
  bytes?: string;
  base64ReadOnly?: string;
  sha256?: string;
  utf8Text?: string;
  close?: string;
  save?: string;
  name?: string;
  kind?: string;
  type?: string;
  location?: string;
  logicalPath?: string;
  revision?: string;
  ok?: string;
  newName?: string;
  renameDialogTitle?: string;
  moveDialogTitle?: string;
  deleteDialogTitle?: string;
  destinationFolderPath?: string;
  emptyForSandboxRoot?: string;
  moveHint?: string;
  move?: string;
  cancel?: string;
  deleteConfirm?: string;
  fileKindNoun?: string;
  directoryKindNoun?: string;
  fileFolderType?: string;
  fileTypeExtension?: string;
  fileTypeGeneric?: string;
  sandboxFolderType?: string;
  loadDirectoryError?: string;
  loadSandboxesError?: string;
  loadMoreSandboxesError?: string;
  loadMoreEntriesError?: string;
  createDirectoryError?: string;
  createFileError?: string;
  readFileError?: string;
  saveFileError?: string;
  renameEntryError?: string;
  moveEntryError?: string;
  deleteEntryError?: string;
  copyPathError?: string;
}

/** 内置英文文案；调用方可通过 labels 覆盖任意键。 */
export const DEFAULT_SANDBOX_EXPLORER_LABELS: Required<SandboxExplorerLabels> = {
  sandboxFileExplorer: 'Sandbox file explorer',
  back: 'Back',
  forward: 'Forward',
  parentDirectory: 'Parent directory',
  refresh: 'Refresh',
  navigationHistory: 'Navigation history',
  currentLogicalPath: 'Current logical path',
  sandboxAbsolutePath: 'Sandbox absolute path',
  noSandboxPath: 'No sandbox path available',
  copyPath: 'Copy path',
  pathCopied: 'Path copied',
  copyFullPath: 'Copy full sandbox path',
  pathCopiedAnnouncement: 'Sandbox path copied to clipboard.',
  filterLoadedItems: 'Filter loaded items',
  searchPlaceholder: 'Filter loaded items in {name}',
  clearSearch: 'Clear search',
  newFolder: 'New folder',
  newFile: 'New file',
  uploadFile: 'Upload file',
  uploadingFiles: 'Uploading {current}/{total}…',
  fileTooLarge: 'File exceeds the 4 MB sandbox limit.',
  uploadError: 'Unable to upload the file.',
  folderName: 'Folder name',
  fileName: 'File name',
  createFolder: 'Create folder',
  createFile: 'Create file',
  cancelFileCreation: 'Cancel file creation',
  rename: 'Rename',
  delete: 'Delete',
  sort: 'Sort',
  sortAscending: 'Sort ascending',
  sortDescending: 'Sort descending',
  switchToGridView: 'Switch to grid view',
  switchToDetailsView: 'Switch to details view',
  view: 'View',
  moreOptions: 'More options',
  gridView: 'Grid view',
  detailsView: 'Details view',
  details: 'Details',
  hideDetailsPane: 'Hide details pane',
  showDetailsPane: 'Show details pane',
  reload: 'Reload',
  retry: 'Retry',
  dismiss: 'Dismiss',
  sandboxNavigation: 'Sandbox navigation',
  sandboxes: 'Sandboxes',
  sandbox: 'Sandbox',
  availableSandboxes: 'Available sandboxes',
  openSandbox: 'Open sandbox {name}',
  moreSandboxes: 'More sandboxes',
  loading: 'Loading',
  loadingFolder: 'Loading folder…',
  loadingFile: 'Loading file…',
  noAccessibleSandboxes: 'No accessible sandboxes.',
  folderEmpty: 'This folder is empty.',
  noSearchMatches: 'No items match your search.',
  directoryItems: 'Directory items',
  itemDetails: 'Item details',
  nameColumn: 'Name',
  typeColumn: 'Type',
  locationColumn: 'Location',
  loadMore: 'Load more',
  loadMoreItems: 'Load more items',
  loadingMoreItems: 'Loading more items…',
  moreItemsAvailable: 'More items available',
  allItemsLoaded: 'All items loaded',
  itemsLoadedOne: '{count} item loaded',
  itemsLoadedMany: '{count} items loaded',
  filteredFrom: 'Filtered from {count}',
  filtering: 'Filtering…',
  refreshing: 'Refreshing…',
  noSandboxSelected: 'No sandbox selected',
  selectDirectory: 'Select directory',
  entryActions: '{name} actions',
  currentFolderActions: 'Current folder actions',
  open: 'Open',
  openFile: 'Open file',
  copyAsPath: 'Copy as path',
  copyPathname: 'Copy pathname',
  copyLocation: 'Copy Location',
  copyCurrentPath: 'Copy current path',
  moveTo: 'Move to…',
  getInfo: 'Get Info',
  properties: 'Properties',
  info: 'Info',
  deletePermanently: 'Delete permanently',
  selectItemDetails: 'Select an item to view its details.',
  editFile: 'Edit {name}',
  closeFile: 'Close file',
  fileContent: 'File content',
  binaryPreview: 'Binary preview',
  bytes: '{count} bytes',
  base64ReadOnly: 'Base64-encoded read-only content',
  sha256: 'SHA-256 {digest}',
  utf8Text: 'UTF-8 text',
  close: 'Close',
  save: 'Save',
  name: 'Name',
  kind: 'Kind',
  type: 'Type',
  location: 'Location',
  logicalPath: 'Logical path',
  revision: 'Revision',
  ok: 'OK',
  newName: 'New name',
  renameDialogTitle: 'Rename {name}',
  moveDialogTitle: 'Move {name}',
  deleteDialogTitle: 'Delete {name}',
  destinationFolderPath: 'Destination folder path',
  emptyForSandboxRoot: 'Empty for sandbox root',
  moveHint: 'Use a sandbox-relative path with forward slashes.',
  move: 'Move',
  cancel: 'Cancel',
  deleteConfirm: 'This permanently deletes the {kind}. This action cannot be undone.',
  fileKindNoun: 'file',
  directoryKindNoun: 'directory',
  fileFolderType: 'File folder',
  fileTypeExtension: '{extension} file',
  fileTypeGeneric: 'File',
  sandboxFolderType: 'Sandbox folder',
  loadDirectoryError: 'Unable to load the sandbox directory.',
  loadSandboxesError: 'Unable to load available sandboxes.',
  loadMoreSandboxesError: 'Unable to load more sandboxes.',
  loadMoreEntriesError: 'Unable to load more directory entries.',
  createDirectoryError: 'Unable to create the directory.',
  createFileError: 'Unable to create the file.',
  readFileError: 'Unable to read the file.',
  saveFileError: 'Unable to save the file.',
  renameEntryError: 'Unable to rename the entry.',
  moveEntryError: 'Unable to move the entry.',
  deleteEntryError: 'Unable to delete the entry.',
  copyPathError: 'Unable to copy the sandbox path.',
};

function interpolateLabels(template: string, values: Readonly<Record<string, string>>): string {
  return template.replace(/\{(\w+)\}/gu, (match, key: string) => values[key] ?? match);
}

export interface SandboxExplorerViewProps {
  readonly mode?: 'manage' | 'select-directory';
  readonly port?: SandboxExplorerPort;
  readonly onDirectorySelected?: (selection: SandboxSelection) => void;
  readonly onDirectoryChanged?: (selection: SandboxSelection) => void;
  readonly className?: string;
  /** 界面文案覆盖；缺省使用内置英文文案。 */
  readonly labels?: SandboxExplorerLabels;
}

function currentSelection(root: SandboxRoot, directory: DirectoryLocation): SandboxSelection {
  const directoryName = directory.logicalPath.split('/').filter(Boolean).at(-1)
    ?? root.displayName;
  return {
    sandboxId: root.id,
    sandboxDisplayName: root.displayName,
    entryId: directory.entryId,
    directoryName,
    logicalPath: directory.logicalPath,
    displayPath: directory.logicalPath
      ? `${root.displayName} / ${directory.logicalPath}`
      : `${root.displayName} /`,
  };
}

function sandboxAbsolutePath(root: SandboxRoot | null, directory: DirectoryLocation | null): string {
  if (!root || !directory) return '';
  return `sandbox://${root.id}/${directory.logicalPath}`;
}

function copyTextFallback(value: string): boolean {
  if (typeof document === 'undefined') return false;
  const input = document.createElement('textarea');
  input.value = value;
  input.setAttribute('readonly', '');
  input.style.position = 'fixed';
  input.style.opacity = '0';
  document.body.append(input);
  input.select();
  const copied = document.execCommand('copy');
  input.remove();
  return copied;
}

function mergeUniqueEntries(
  current: readonly SandboxEntry[],
  incoming: readonly SandboxEntry[],
): readonly SandboxEntry[] {
  const known = new Set(current.map((entry) => entry.id));
  return [...current, ...incoming.filter((entry) => !known.has(entry.id))];
}

function buildBreadcrumbs(
  root: SandboxRoot | null,
  directory: DirectoryLocation | null,
  entryIdsByPath: ReadonlyMap<string, string>,
): readonly BreadcrumbItem[] {
  if (!root || !directory) return [];
  const breadcrumbs: BreadcrumbItem[] = [
    { label: root.displayName, logicalPath: '', entryId: root.rootEntryId },
  ];
  let logicalPath = '';
  for (const segment of directory.logicalPath.split('/').filter(Boolean)) {
    logicalPath = logicalPath ? `${logicalPath}/${segment}` : segment;
    const entryId = entryIdsByPath.get(logicalPath);
    if (entryId) breadcrumbs.push({ label: segment, logicalPath, entryId });
  }
  return breadcrumbs;
}

function errorMessage(cause: unknown, fallback: string): string {
  return cause instanceof Error && cause.message.trim() ? cause.message : fallback;
}

function entryType(entry: SandboxEntry, labels: Required<SandboxExplorerLabels>): string {
  if (entry.kind === 'directory') return labels.fileFolderType;
  const extension = entry.name.includes('.') ? entry.name.split('.').at(-1) : undefined;
  return extension
    ? interpolateLabels(labels.fileTypeExtension, { extension: extension.toUpperCase() })
    : labels.fileTypeGeneric;
}

function entryLocation(entry: SandboxEntry): string {
  const segments = entry.logicalPath.split('/').filter(Boolean);
  return segments.slice(0, -1).join('/') || '/';
}

const TEXT_FILE_EXTENSIONS = new Set([
  'c', 'conf', 'cpp', 'cs', 'css', 'csv', 'dart', 'env', 'go', 'h', 'html', 'ini', 'java',
  'js', 'json', 'jsx', 'kt', 'less', 'log', 'md', 'mjs', 'php', 'properties', 'py', 'rb',
  'rs', 'scss', 'sh', 'sql', 'svg', 'swift', 'toml', 'ts', 'tsx', 'txt', 'xml', 'yaml', 'yml',
]);

function preferredFileEncoding(name: string): 'utf8' | 'base64' {
  const extension = name.includes('.') ? name.split('.').at(-1)?.toLocaleLowerCase() : undefined;
  return extension && TEXT_FILE_EXTENSIONS.has(extension) ? 'utf8' : 'base64';
}

function readFileAsBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onerror = () => reject(reader.error ?? new Error('File read failed.'));
    reader.onload = () => {
      const result = typeof reader.result === 'string' ? reader.result : '';
      const comma = result.indexOf(',');
      resolve(comma >= 0 ? result.slice(comma + 1) : result);
    };
    reader.readAsDataURL(file);
  });
}

export function SandboxExplorerView({
  mode = 'manage',
  port: injectedPort,
  onDirectorySelected,
  onDirectoryChanged,
  className,
  labels: injectedLabels,
}: SandboxExplorerViewProps) {
  const labels = useMemo(
    () => ({ ...DEFAULT_SANDBOX_EXPLORER_LABELS, ...injectedLabels }),
    [injectedLabels],
  );
  const port = useMemo(
    () => injectedPort ?? requireDriveSandboxExplorerPort(),
    [injectedPort],
  );
  const sandboxSelectId = useId();
  const newDirectoryNameId = useId();
  const searchId = useId();
  const explorerRef = useRef<HTMLElement>(null);
  const loadMoreRef = useRef<HTMLButtonElement>(null);
  const addressInputRef = useRef<HTMLInputElement>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const moreMenuRef = useRef<HTMLDivElement>(null);
  const contextMenuRef = useRef<HTMLDivElement>(null);
  const contentRef = useRef<HTMLElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const requestSequence = useRef(0);
  const loadingMoreRequestId = useRef<number | null>(null);
  const automaticLoadingPaused = useRef(false);
  const currentRootId = useRef<string | null>(null);
  const currentDirectoryKey = useRef<string | null>(null);
  const pendingFocusEntryId = useRef<string | null>(null);
  const selectedEntryIdRef = useRef<string | null>(null);
  const historyIndexRef = useRef(-1);
  const entryIdsByPath = useRef(new Map<string, string>());
  const entryElementsById = useRef(new Map<string, HTMLButtonElement>());
  const onDirectoryChangedRef = useRef(onDirectoryChanged);
  const copyFeedbackTimerRef = useRef<ReturnType<typeof globalThis.setTimeout> | null>(null);
  const typeaheadBufferRef = useRef('');
  const typeaheadTimerRef = useRef<ReturnType<typeof globalThis.setTimeout> | null>(null);
  const [roots, setRoots] = useState<readonly SandboxRoot[]>([]);
  const [root, setRoot] = useState<SandboxRoot | null>(null);
  const [directory, setDirectory] = useState<DirectoryLocation | null>(null);
  const [entries, setEntries] = useState<readonly SandboxEntry[]>([]);
  const [selectedEntry, setSelectedEntry] = useState<SandboxEntry | null>(null);
  const [nextCursor, setNextCursor] = useState<string | undefined>();
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [sandboxPage, setSandboxPage] = useState(1);
  const [sandboxTotalPages, setSandboxTotalPages] = useState(1);
  const [sandboxLoadAttempt, setSandboxLoadAttempt] = useState(0);
  const [loadingMoreSandboxes, setLoadingMoreSandboxes] = useState(false);
  const [creatingDirectory, setCreatingDirectory] = useState(false);
  const [newDirectoryName, setNewDirectoryName] = useState('');
  const [creatingFile, setCreatingFile] = useState(false);
  const [newFileName, setNewFileName] = useState('');
  const [createPending, setCreatePending] = useState(false);
  const [uploadProgress, setUploadProgress] = useState<{ current: number; total: number } | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const deferredSearchQuery = useDeferredValue(searchQuery);
  const [addressFocused, setAddressFocused] = useState(false);
  const [pathCopied, setPathCopied] = useState(false);
  const [viewMode, setViewMode] = useState<ViewMode>('details');
  const [sortAscending, setSortAscending] = useState(true);
  const [detailsVisible, setDetailsVisible] = useState(true);
  const [moreMenuOpen, setMoreMenuOpen] = useState(false);
  const [contextMenu, setContextMenu] = useState<EntryContextMenu | null>(null);
  const [fileEditor, setFileEditor] = useState<FileEditorState | null>(null);
  const [renamingEntry, setRenamingEntry] = useState<SandboxEntry | null>(null);
  const [renameValue, setRenameValue] = useState('');
  const [movingEntry, setMovingEntry] = useState<SandboxEntry | null>(null);
  const [moveDestination, setMoveDestination] = useState('');
  const [deletingEntry, setDeletingEntry] = useState<SandboxEntry | null>(null);
  const [propertiesTarget, setPropertiesTarget] = useState<PropertiesTarget | null>(null);
  const [mutationPending, setMutationPending] = useState(false);
  const [history, setHistory] = useState<readonly HistoryLocation[]>([]);
  const [historyIndex, setHistoryIndex] = useState(-1);
  const platform = resolveDesktopPlatform();

  useEffect(() => {
    onDirectoryChangedRef.current = onDirectoryChanged;
  }, [onDirectoryChanged]);

  useEffect(() => {
    selectedEntryIdRef.current = selectedEntry?.id ?? null;
  }, [selectedEntry]);

  useEffect(() => {
    if (!moreMenuOpen) return undefined;
    const dismissMenu = (event: PointerEvent) => {
      if (!moreMenuRef.current?.contains(event.target as Node)) setMoreMenuOpen(false);
    };
    const dismissMenuFromKeyboard = (event: KeyboardEvent) => {
      if (event.key === 'Escape') setMoreMenuOpen(false);
    };
    document.addEventListener('pointerdown', dismissMenu);
    document.addEventListener('keydown', dismissMenuFromKeyboard);
    return () => {
      document.removeEventListener('pointerdown', dismissMenu);
      document.removeEventListener('keydown', dismissMenuFromKeyboard);
    };
  }, [moreMenuOpen]);

  useEffect(() => {
    if (!contextMenu) return undefined;
    const dismissMenu = (event: PointerEvent) => {
      if (!contextMenuRef.current?.contains(event.target as Node)) {
        setContextMenu(null);
      }
    };
    const dismissMenuFromKeyboard = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        setContextMenu(null);
        contextMenu.returnFocus?.focus();
      }
    };
    document.addEventListener('pointerdown', dismissMenu);
    document.addEventListener('keydown', dismissMenuFromKeyboard);
    return () => {
      document.removeEventListener('pointerdown', dismissMenu);
      document.removeEventListener('keydown', dismissMenuFromKeyboard);
    };
  }, [contextMenu]);

  useLayoutEffect(() => {
    const menu = contextMenuRef.current;
    if (!contextMenu || !menu) return;
    const bounds = menu.getBoundingClientRect();
    const x = Math.max(8, Math.min(contextMenu.x, globalThis.innerWidth - bounds.width - 8));
    const y = Math.max(8, Math.min(contextMenu.y, globalThis.innerHeight - bounds.height - 8));
    if (x === contextMenu.x && y === contextMenu.y) return;
    setContextMenu((current) => current ? { ...current, x, y } : null);
  }, [contextMenu]);

  useEffect(() => {
    if (!contextMenu?.focusMenu) return;
    const firstItem = contextMenuRef.current?.querySelector<HTMLButtonElement>(
      'button[role^="menuitem"]:not(:disabled)',
    );
    firstItem?.focus();
  }, [contextMenu]);

  const rememberEntries = useCallback((items: readonly SandboxEntry[]) => {
    for (const entry of items) entryIdsByPath.current.set(entry.logicalPath, entry.id);
  }, []);

  const loadDirectory = useCallback(async (
    nextRoot: SandboxRoot,
    nextDirectory: DirectoryLocation,
    historyMode: HistoryMode = 'push',
  ) => {
    const requestId = ++requestSequence.current;
    const directoryKey = `${nextRoot.id}\u0000${nextDirectory.logicalPath}`;
    const refreshInPlace = currentDirectoryKey.current === directoryKey;
    pendingFocusEntryId.current = refreshInPlace && document.activeElement instanceof HTMLElement
      ? document.activeElement.closest<HTMLElement>('[data-entry-id]')?.dataset.entryId
        ?? selectedEntryIdRef.current
      : null;
    loadingMoreRequestId.current = null;
    automaticLoadingPaused.current = false;
    if (refreshInPlace) {
      setRefreshing(true);
    } else {
      setRefreshing(false);
      setLoading(true);
    }
    setLoadingMore(false);
    setError(null);
    setCreatingDirectory(false);
    setCreatingFile(false);
    setContextMenu(null);
    try {
      const page = await port.listChildren({
        sandboxId: nextRoot.id,
        parentPath: nextDirectory.logicalPath,
        pageSize: DIRECTORY_PAGE_SIZE,
      });
      if (requestSequence.current !== requestId) return;
      if (currentRootId.current !== nextRoot.id) entryIdsByPath.current.clear();
      currentRootId.current = nextRoot.id;
      currentDirectoryKey.current = directoryKey;
      entryIdsByPath.current.set('', nextRoot.rootEntryId);
      entryIdsByPath.current.set(nextDirectory.logicalPath, nextDirectory.entryId);
      rememberEntries(page.items);
      setRoot(nextRoot);
      setDirectory(nextDirectory);
      setEntries(page.items);
      setSelectedEntry((current) => refreshInPlace && current
        ? page.items.find((entry) => entry.id === current.id) ?? null
        : null);
      if (!refreshInPlace) setSearchQuery('');
      setNextCursor(page.nextCursor);
      if (historyMode !== 'preserve') {
        const location: HistoryLocation = { rootId: nextRoot.id, directory: nextDirectory };
        setHistory((current) => {
          if (historyMode === 'replace') return [location];
          const retained = current.slice(0, historyIndexRef.current + 1);
          const last = retained.at(-1);
          if (
            last?.rootId === location.rootId
            && last.directory.logicalPath === location.directory.logicalPath
          ) {
            return retained;
          }
          return [...retained, location];
        });
        setHistoryIndex((current) => {
          const nextIndex = historyMode === 'replace' ? 0 : current + 1;
          historyIndexRef.current = nextIndex;
          return nextIndex;
        });
      }
      onDirectoryChangedRef.current?.(currentSelection(nextRoot, nextDirectory));
    } catch (cause) {
      if (requestSequence.current === requestId) {
        setError(errorMessage(cause, labels.loadDirectoryError));
      }
    } finally {
      if (requestSequence.current === requestId) {
        setLoading(false);
        setRefreshing(false);
      }
    }
  }, [port, rememberEntries]);

  useEffect(() => {
    let active = true;
    setLoading(true);
    void port.listSandboxes({ page: 1, pageSize: SANDBOX_PAGE_SIZE })
      .then((result) => {
        if (!active) return;
        setRoots(result.items);
        setSandboxPage(result.page);
        setSandboxTotalPages(Math.max(result.totalPages, 1));
        const first = result.items[0];
        if (!first) {
          setLoading(false);
          return;
        }
        void loadDirectory(
          first,
          { entryId: first.rootEntryId, logicalPath: '' },
          'replace',
        );
      })
      .catch((cause) => {
        if (!active) return;
        setError(errorMessage(cause, labels.loadSandboxesError));
        setLoading(false);
      });
    return () => {
      active = false;
      requestSequence.current += 1;
    };
  }, [loadDirectory, port, sandboxLoadAttempt]);

  const breadcrumbs = useMemo(
    () => buildBreadcrumbs(root, directory, entryIdsByPath.current),
    [directory, entries, root],
  );
  const absolutePath = useMemo(
    () => sandboxAbsolutePath(root, directory),
    [directory, root],
  );

  useEffect(() => {
    if (!addressFocused) return;
    addressInputRef.current?.focus();
    addressInputRef.current?.select();
  }, [absolutePath, addressFocused]);

  useEffect(() => {
    setPathCopied(false);
  }, [absolutePath]);

  useEffect(() => () => {
    if (copyFeedbackTimerRef.current) globalThis.clearTimeout(copyFeedbackTimerRef.current);
    if (typeaheadTimerRef.current) globalThis.clearTimeout(typeaheadTimerRef.current);
  }, []);

  const visibleEntries = useMemo(() => {
    const query = deferredSearchQuery.trim().toLocaleLowerCase();
    return entries
      .filter((entry) => !query || entry.name.toLocaleLowerCase().includes(query))
      .slice()
      .sort((left, right) => {
        if (left.kind !== right.kind) return left.kind === 'directory' ? -1 : 1;
        const order = DIRECTORY_NAME_COLLATOR.compare(left.name, right.name);
        return sortAscending ? order : -order;
      });
  }, [deferredSearchQuery, entries, sortAscending]);
  const selectedVisibleIndex = selectedEntry
    ? visibleEntries.findIndex((entry) => entry.id === selectedEntry.id)
    : -1;
  const rovingEntryId = selectedVisibleIndex >= 0
    ? selectedEntry?.id
    : visibleEntries[0]?.id;
  const filtering = searchQuery !== deferredSearchQuery;

  useEffect(() => {
    if (selectedEntry && selectedVisibleIndex < 0) setSelectedEntry(null);
  }, [selectedEntry, selectedVisibleIndex]);

  useLayoutEffect(() => {
    if (refreshing) return;
    const entryId = pendingFocusEntryId.current;
    if (!entryId) return;
    const target = entryElementsById.current.get(entryId);
    if (!target) {
      if (!entries.some((entry) => entry.id === entryId)) {
        pendingFocusEntryId.current = null;
      }
      return;
    }
    target.focus({ preventScroll: true });
    if (document.activeElement === target) {
      pendingFocusEntryId.current = null;
    }
  }, [entries, refreshing]);

  const loadMoreSandboxes = async () => {
    if (loadingMoreSandboxes || sandboxPage >= sandboxTotalPages) return;
    setLoadingMoreSandboxes(true);
    setError(null);
    try {
      const result = await port.listSandboxes({
        page: sandboxPage + 1,
        pageSize: SANDBOX_PAGE_SIZE,
      });
      setRoots((current) => {
        const known = new Set(current.map((item) => item.id));
        return [...current, ...result.items.filter((item) => !known.has(item.id))];
      });
      setSandboxPage(result.page);
      setSandboxTotalPages(Math.max(result.totalPages, 1));
    } catch (cause) {
      setError(errorMessage(cause, labels.loadMoreSandboxesError));
    } finally {
      setLoadingMoreSandboxes(false);
    }
  };

  const loadMoreEntries = useCallback(async () => {
    if (!root || !directory || !nextCursor) return;
    const requestId = requestSequence.current;
    if (loadingMoreRequestId.current === requestId) return;
    loadingMoreRequestId.current = requestId;
    setLoadingMore(true);
    setError(null);
    try {
      const page = await port.listChildren({
        sandboxId: root.id,
        parentPath: directory.logicalPath,
        cursor: nextCursor,
        pageSize: DIRECTORY_PAGE_SIZE,
      });
      if (requestSequence.current !== requestId) return;
      rememberEntries(page.items);
      setEntries((current) => mergeUniqueEntries(current, page.items));
      setNextCursor(page.nextCursor);
      automaticLoadingPaused.current = false;
    } catch (cause) {
      if (requestSequence.current === requestId) {
        automaticLoadingPaused.current = true;
        setError(errorMessage(cause, labels.loadMoreEntriesError));
      }
    } finally {
      if (loadingMoreRequestId.current === requestId) {
        loadingMoreRequestId.current = null;
        setLoadingMore(false);
      }
    }
  }, [directory, nextCursor, port, rememberEntries, root]);

  useEffect(() => {
    const target = loadMoreRef.current;
    if (
      !target
      || loading
      || loadingMore
      || !nextCursor
      || automaticLoadingPaused.current
      || typeof IntersectionObserver === 'undefined'
    ) {
      return undefined;
    }
    const observer = new IntersectionObserver((records) => {
      if (!records.some((record) => record.isIntersecting)) return;
      observer.disconnect();
      void loadMoreEntries();
    }, {
      root: target.closest('.sdkwork-sandbox-explorer__content'),
      rootMargin: '200px 0px',
    });
    observer.observe(target);
    return () => observer.disconnect();
  }, [loadMoreEntries, loading, loadingMore, nextCursor]);

  const submitCreateDirectory = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const name = newDirectoryName.trim();
    if (!root || !directory || !name || !root.capabilities.createDirectory || createPending) {
      return;
    }
    setCreatePending(true);
    setError(null);
    const requestId = requestSequence.current;
    try {
      await port.createDirectory({
        sandboxId: root.id,
        parentPath: directory.logicalPath,
        name,
      });
      setNewDirectoryName('');
      setCreatingDirectory(false);
      if (requestSequence.current === requestId) {
        await loadDirectory(root, directory, 'preserve');
      }
    } catch (cause) {
      setError(errorMessage(cause, labels.createDirectoryError));
    } finally {
      setCreatePending(false);
    }
  };

  const submitCreateFile = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const name = newFileName.trim();
    if (!root || !directory || !name || !root.capabilities.createFile || createPending) return;
    setCreatePending(true);
    setError(null);
    try {
      const entry = await port.createFile({
        sandboxId: root.id,
        parentPath: directory.logicalPath,
        name,
        content: '',
        encoding: 'utf8',
      });
      setNewFileName('');
      setCreatingFile(false);
      await loadDirectory(root, directory, 'preserve');
      setSelectedEntry(entry);
    } catch (cause) {
      setError(errorMessage(cause, labels.createFileError));
    } finally {
      setCreatePending(false);
    }
  };

  const handleUploadFiles = async (files: FileList | null) => {
    if (!root || !directory || !files || files.length === 0 || !root.capabilities.createFile || createPending) {
      return;
    }
    const targets = Array.from(files);
    const oversized = targets.find((file) => file.size > SANDBOX_UPLOAD_MAX_BYTES);
    if (oversized) {
      setError(errorMessage(new Error(labels.fileTooLarge), labels.fileTooLarge));
      if (fileInputRef.current) fileInputRef.current.value = '';
      return;
    }
    setCreatePending(true);
    setError(null);
    setUploadProgress({ current: 0, total: targets.length });
    try {
      for (let index = 0; index < targets.length; index += 1) {
        const file = targets[index];
        setUploadProgress({ current: index + 1, total: targets.length });
        const content = await readFileAsBase64(file);
        await port.createFile({
          sandboxId: root.id,
          parentPath: directory.logicalPath,
          name: file.name,
          content,
          encoding: 'base64',
        });
      }
      await loadDirectory(root, directory, 'preserve');
    } catch (cause) {
      setError(errorMessage(cause, labels.uploadError));
    } finally {
      setCreatePending(false);
      setUploadProgress(null);
      if (fileInputRef.current) fileInputRef.current.value = '';
    }
  };

  const navigateUp = () => {
    if (!root || breadcrumbs.length < 2) return;
    const parent = breadcrumbs[breadcrumbs.length - 2];
    if (parent) void loadDirectory(root, parent);
  };

  const navigateHistory = (nextIndex: number) => {
    const location = history[nextIndex];
    const nextRoot = roots.find((candidate) => candidate.id === location?.rootId);
    if (!location || !nextRoot) return;
    historyIndexRef.current = nextIndex;
    setHistoryIndex(nextIndex);
    void loadDirectory(nextRoot, location.directory, 'preserve');
  };

  const selectEntry = (entry: SandboxEntry) => {
    setSelectedEntry(entry);
    setContextMenu(null);
  };

  const focusVisibleEntry = useCallback((index: number) => {
    const boundedIndex = Math.max(0, Math.min(index, visibleEntries.length - 1));
    const entry = visibleEntries[boundedIndex];
    if (!entry) return;
    setSelectedEntry(entry);
    const target = entryElementsById.current.get(entry.id);
    target?.focus({ preventScroll: true });
    target?.scrollIntoView?.({ block: 'nearest', inline: 'nearest' });
  }, [visibleEntries]);

  const handleEntryNavigationKeyDown = (event: React.KeyboardEvent<HTMLElement>) => {
    if (visibleEntries.length === 0) return false;
    const currentIndex = selectedVisibleIndex >= 0 ? selectedVisibleIndex : 0;
    const grid = contentRef.current?.querySelector<HTMLElement>(
      '.sdkwork-sandbox-explorer__grid-view',
    );
    const gridColumns = viewMode === 'grid'
      ? Math.max(1, Math.floor(((grid?.clientWidth ?? 112) + 8) / 120))
      : 1;
    let nextIndex: number | null = null;
    if (event.key === 'Home') nextIndex = 0;
    if (event.key === 'End') nextIndex = visibleEntries.length - 1;
    if (event.key === 'PageUp') nextIndex = currentIndex - 10;
    if (event.key === 'PageDown') nextIndex = currentIndex + 10;
    if (event.key === 'ArrowUp') nextIndex = currentIndex - gridColumns;
    if (event.key === 'ArrowDown') nextIndex = currentIndex + gridColumns;
    if (viewMode === 'grid' && event.key === 'ArrowLeft') nextIndex = currentIndex - 1;
    if (viewMode === 'grid' && event.key === 'ArrowRight') nextIndex = currentIndex + 1;
    if (nextIndex !== null) {
      event.preventDefault();
      focusVisibleEntry(nextIndex);
      return true;
    }
    if (
      event.key.length !== 1
      || event.ctrlKey
      || event.metaKey
      || event.altKey
      || /^\s$/u.test(event.key)
    ) {
      return false;
    }
    typeaheadBufferRef.current += event.key.toLocaleLowerCase();
    if (typeaheadTimerRef.current) globalThis.clearTimeout(typeaheadTimerRef.current);
    typeaheadTimerRef.current = globalThis.setTimeout(() => {
      typeaheadBufferRef.current = '';
    }, TYPEAHEAD_RESET_DELAY_MS);
    let matchIndex = -1;
    for (let offset = 1; offset <= visibleEntries.length; offset += 1) {
      const candidateIndex = (currentIndex + offset) % visibleEntries.length;
      const candidate = visibleEntries[candidateIndex];
      if (candidate?.name.toLocaleLowerCase().startsWith(typeaheadBufferRef.current)) {
        matchIndex = candidateIndex;
        break;
      }
    }
    if (matchIndex >= 0) {
      event.preventDefault();
      focusVisibleEntry(matchIndex);
      return true;
    }
    return false;
  };

  const openFile = async (entry: SandboxEntry) => {
    if (!root?.capabilities.readFile) return;
    const encoding = preferredFileEncoding(entry.name);
    setFileEditor({
      entry,
      content: '',
      encoding,
      sizeBytes: '0',
      checksumSha256: '',
      loading: true,
      saving: false,
      error: null,
    });
    try {
      const content = await port.readFile({
        sandboxId: root.id,
        entryId: entry.id,
        logicalPath: entry.logicalPath,
        encoding,
      });
      setFileEditor({
        entry: content.entry,
        content: content.content,
        encoding: content.encoding,
        sizeBytes: content.sizeBytes,
        checksumSha256: content.checksumSha256,
        loading: false,
        saving: false,
        error: null,
      });
      setEntries((current) => current.map((item) => item.id === entry.id ? content.entry : item));
      setSelectedEntry(content.entry);
    } catch (cause) {
      setFileEditor((current) => current ? {
        ...current,
        loading: false,
        error: errorMessage(cause, labels.readFileError),
      } : null);
    }
  };

  const activateEntry = (entry: SandboxEntry) => {
    setSelectedEntry(entry);
    if (root && entry.kind === 'directory') {
      void loadDirectory(root, { entryId: entry.id, logicalPath: entry.logicalPath });
    } else if (mode === 'manage' && entry.kind === 'file') {
      void openFile(entry);
    }
  };

  const saveFile = async () => {
    if (!root || !fileEditor || !root.capabilities.writeFile || fileEditor.saving) return;
    setFileEditor((current) => current ? { ...current, saving: true, error: null } : null);
    try {
      const entry = await port.updateFile({
        sandboxId: root.id,
        entryId: fileEditor.entry.id,
        logicalPath: fileEditor.entry.logicalPath,
        revision: fileEditor.entry.revision,
        content: fileEditor.content,
        encoding: fileEditor.encoding,
      });
      setFileEditor((current) => current ? { ...current, entry, saving: false } : null);
      setEntries((current) => current.map((item) => item.id === entry.id ? entry : item));
      setSelectedEntry(entry);
    } catch (cause) {
      setFileEditor((current) => current ? {
        ...current,
        saving: false,
        error: errorMessage(cause, labels.saveFileError),
      } : null);
    }
  };

  const startRename = (entry: SandboxEntry) => {
    setContextMenu(null);
    setRenamingEntry(entry);
    setRenameValue(entry.name);
  };

  const submitRename = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const destinationName = renameValue.trim();
    if (!root || !directory || !renamingEntry || !destinationName || mutationPending) return;
    setMutationPending(true);
    setError(null);
    try {
      const entry = await port.moveEntry({
        sandboxId: root.id,
        entryId: renamingEntry.id,
        logicalPath: renamingEntry.logicalPath,
        revision: renamingEntry.revision,
        destinationParentPath: entryLocation(renamingEntry) === '/' ? '' : entryLocation(renamingEntry),
        destinationName,
      });
      setRenamingEntry(null);
      await loadDirectory(root, directory, 'preserve');
      setSelectedEntry(entry);
    } catch (cause) {
      setError(errorMessage(cause, labels.renameEntryError));
    } finally {
      setMutationPending(false);
    }
  };

  const startMove = (entry: SandboxEntry) => {
    setContextMenu(null);
    setMovingEntry(entry);
    setMoveDestination(entryLocation(entry) === '/' ? '' : entryLocation(entry));
  };

  const submitMove = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!root || !directory || !movingEntry || mutationPending) return;
    setMutationPending(true);
    setError(null);
    try {
      const entry = await port.moveEntry({
        sandboxId: root.id,
        entryId: movingEntry.id,
        logicalPath: movingEntry.logicalPath,
        revision: movingEntry.revision,
        destinationParentPath: moveDestination.trim(),
        destinationName: movingEntry.name,
      });
      setMovingEntry(null);
      await loadDirectory(root, directory, 'preserve');
      setSelectedEntry(entry);
    } catch (cause) {
      setError(errorMessage(cause, labels.moveEntryError));
    } finally {
      setMutationPending(false);
    }
  };

  const confirmDelete = async () => {
    if (!root || !directory || !deletingEntry || mutationPending) return;
    setMutationPending(true);
    setError(null);
    try {
      await port.deleteEntry({
        sandboxId: root.id,
        entryId: deletingEntry.id,
        logicalPath: deletingEntry.logicalPath,
        revision: deletingEntry.revision,
        recursive: deletingEntry.kind === 'directory',
      });
      setDeletingEntry(null);
      await loadDirectory(root, directory, 'preserve');
    } catch (cause) {
      setError(errorMessage(cause, labels.deleteEntryError));
    } finally {
      setMutationPending(false);
    }
  };

  const selectCurrentDirectory = () => {
    if (!root || !directory || !root.capabilities.selectDirectory) return;
    onDirectorySelected?.(currentSelection(root, directory));
  };

  const copyCurrentPath = async () => {
    if (!absolutePath) return;
    try {
      let copied = false;
      if (navigator.clipboard?.writeText) {
        try {
          await navigator.clipboard.writeText(absolutePath);
          copied = true;
        } catch {
          copied = false;
        }
      }
      if (!copied) copied = copyTextFallback(absolutePath);
      if (!copied) throw new Error('Clipboard access is unavailable.');
      setPathCopied(true);
      if (copyFeedbackTimerRef.current) globalThis.clearTimeout(copyFeedbackTimerRef.current);
      copyFeedbackTimerRef.current = globalThis.setTimeout(() => setPathCopied(false), 1800);
    } catch (cause) {
      setError(errorMessage(cause, labels.copyPathError));
    }
  };

  const copyPath = async (value: string) => {
    try {
      let copied = false;
      if (navigator.clipboard?.writeText) {
        try {
          await navigator.clipboard.writeText(value);
          copied = true;
        } catch {
          copied = false;
        }
      }
      if (!copied) copied = copyTextFallback(value);
      if (!copied) throw new Error('Clipboard access is unavailable.');
      setPathCopied(true);
      if (copyFeedbackTimerRef.current) globalThis.clearTimeout(copyFeedbackTimerRef.current);
      copyFeedbackTimerRef.current = globalThis.setTimeout(() => setPathCopied(false), 1800);
    } catch (cause) {
      setError(errorMessage(cause, labels.copyPathError));
    }
  };

  const entryAbsolutePath = (entry: SandboxEntry): string => (
    root ? `sandbox://${root.id}/${entry.logicalPath}` : ''
  );

  const openEntryContextMenu = (
    entry: SandboxEntry,
    x: number,
    y: number,
    returnFocus: HTMLElement | null,
    focusMenu = false,
  ) => {
    setMoreMenuOpen(false);
    selectEntry(entry);
    setContextMenu({ kind: 'entry', entry, x, y, focusMenu, returnFocus });
  };

  const openBackgroundContextMenu = (
    x: number,
    y: number,
    returnFocus: HTMLElement | null,
    focusMenu = false,
  ) => {
    setMoreMenuOpen(false);
    setSelectedEntry(null);
    setContextMenu({ kind: 'background', x, y, focusMenu, returnFocus });
  };

  const handleContextMenuKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    const menuItems = Array.from(
      event.currentTarget.querySelectorAll<HTMLButtonElement>('button[role^="menuitem"]:not(:disabled)'),
    );
    if (menuItems.length === 0) return;
    const currentIndex = menuItems.findIndex((item) => item === document.activeElement);
    let nextIndex: number | null = null;
    if (event.key === 'ArrowDown') nextIndex = currentIndex < 0 ? 0 : (currentIndex + 1) % menuItems.length;
    if (event.key === 'ArrowUp') nextIndex = currentIndex < 0 ? menuItems.length - 1 : (currentIndex - 1 + menuItems.length) % menuItems.length;
    if (event.key === 'Home') nextIndex = 0;
    if (event.key === 'End') nextIndex = menuItems.length - 1;
    if (event.key === 'Tab') {
      setContextMenu(null);
      contextMenu?.returnFocus?.focus();
      return;
    }
    if (event.key.length === 1 && !event.ctrlKey && !event.metaKey && !event.altKey) {
      const query = event.key.toLocaleLowerCase();
      const startIndex = currentIndex < 0 ? 0 : currentIndex + 1;
      let match: HTMLButtonElement | undefined;
      for (let offset = 0; offset < menuItems.length; offset += 1) {
        const candidate = menuItems[(startIndex + offset) % menuItems.length];
        if (candidate?.textContent?.trim().toLocaleLowerCase().startsWith(query)) {
          match = candidate;
          break;
        }
      }
      if (match) {
        event.preventDefault();
        match.focus();
      }
      return;
    }
    if (nextIndex === null) return;
    event.preventDefault();
    menuItems[nextIndex]?.focus();
  };

  const currentLabel = breadcrumbs.at(-1)?.label ?? root?.displayName ?? labels.sandbox;
  const detailName = selectedEntry?.name ?? currentLabel;
  const detailKind = selectedEntry ? entryType(selectedEntry, labels) : labels.sandboxFolderType;
  const explorerClassName = [
    'sdkwork-sandbox-explorer',
    error ? 'has-error' : '',
    className,
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <section
      ref={explorerRef}
      className={explorerClassName}
      aria-label={labels.sandboxFileExplorer}
      onKeyDown={(event) => {
        if ((event.ctrlKey || event.metaKey) && event.key.toLocaleLowerCase() === 'f') {
          event.preventDefault();
          searchInputRef.current?.focus();
          return;
        }
        const target = event.target;
        if (
          target instanceof HTMLInputElement
          || target instanceof HTMLSelectElement
          || target instanceof HTMLTextAreaElement
          || (target instanceof HTMLElement && target.isContentEditable)
          || (target instanceof HTMLElement && Boolean(target.closest(
            '[role="menu"], .sdkwork-sandbox-explorer__modal-backdrop',
          )))
        ) return;
        if (
          target instanceof HTMLElement
          && target.closest('[data-entry-id]')
          && handleEntryNavigationKeyDown(event)
        ) return;
        if (event.altKey && event.key === 'ArrowLeft' && historyIndex > 0) {
          event.preventDefault();
          navigateHistory(historyIndex - 1);
        } else if (
          event.altKey
          && event.key === 'ArrowRight'
          && historyIndex >= 0
          && historyIndex < history.length - 1
        ) {
          event.preventDefault();
          navigateHistory(historyIndex + 1);
        } else if (event.key === 'Backspace' && directory?.logicalPath) {
          event.preventDefault();
          navigateUp();
        } else if (event.key === 'F5' && root && directory && !loading && !refreshing) {
          event.preventDefault();
          void loadDirectory(root, directory, 'preserve');
        } else if (event.key === 'Enter' && selectedEntry) {
          event.preventDefault();
          activateEntry(selectedEntry);
        } else if (mode === 'manage' && event.key === 'F2' && selectedEntry && root?.capabilities.moveEntry) {
          event.preventDefault();
          startRename(selectedEntry);
        } else if (mode === 'manage' && event.key === 'Delete' && selectedEntry && root?.capabilities.deleteEntry) {
          event.preventDefault();
          setDeletingEntry(selectedEntry);
        } else if ((event.shiftKey && event.key === 'F10') || event.key === 'ContextMenu') {
          event.preventDefault();
          const returnFocus = document.activeElement instanceof HTMLElement
            ? document.activeElement
            : explorerRef.current;
          const entryTarget = event.target instanceof HTMLElement
            ? event.target.closest<HTMLElement>('[data-entry-id]')
            : null;
          const targetEntry = entries.find((entry) => entry.id === entryTarget?.dataset.entryId)
            ?? selectedEntry;
          const bounds = targetEntry
            ? entryTarget?.getBoundingClientRect()
            : contentRef.current?.getBoundingClientRect();
          const x = bounds ? bounds.left + Math.min(36, bounds.width / 2) : 16;
          const y = bounds ? bounds.top + Math.min(28, bounds.height) : 80;
          if (targetEntry) {
            openEntryContextMenu(targetEntry, x, y, returnFocus, true);
          } else {
            openBackgroundContextMenu(x, y, returnFocus, true);
          }
        }
      }}
    >
      <div className="sdkwork-sandbox-explorer__navigation">
        <div className="sdkwork-sandbox-explorer__history" role="group" aria-label={labels.navigationHistory}>
          <button
            type="button"
            title={labels.back}
            aria-label={labels.back}
            className="sdkwork-sandbox-explorer__icon-button"
            disabled={historyIndex <= 0 || loading}
            onClick={() => navigateHistory(historyIndex - 1)}
          >
            <ArrowLeft size={16} />
          </button>
          <button
            type="button"
            title={labels.forward}
            aria-label={labels.forward}
            className="sdkwork-sandbox-explorer__icon-button"
            disabled={historyIndex < 0 || historyIndex >= history.length - 1 || loading}
            onClick={() => navigateHistory(historyIndex + 1)}
          >
            <ArrowRight size={16} />
          </button>
          <button
            type="button"
            title={labels.parentDirectory}
            aria-label={labels.parentDirectory}
            className="sdkwork-sandbox-explorer__icon-button"
            disabled={!directory?.logicalPath || loading}
            onClick={navigateUp}
          >
            <ArrowUp size={16} />
          </button>
          <button
            type="button"
            title={labels.refresh}
            aria-label={labels.refresh}
            className="sdkwork-sandbox-explorer__icon-button"
            disabled={!root || !directory || loading || refreshing}
            onClick={() => root && directory && void loadDirectory(root, directory, 'preserve')}
          >
            <RefreshCw size={15} className={loading || refreshing ? 'is-spinning' : undefined} />
          </button>
        </div>

        <nav
          className={`sdkwork-sandbox-explorer__address${addressFocused ? ' is-focused' : ''}`}
          aria-label={labels.currentLogicalPath}
          tabIndex={0}
          title={absolutePath || labels.noSandboxPath}
          onClick={(event) => {
            if ((event.target as HTMLElement).closest('.sdkwork-sandbox-explorer__address-copy')) return;
            setAddressFocused(true);
          }}
          onFocus={(event) => {
            if (event.target === event.currentTarget) setAddressFocused(true);
          }}
          onBlur={(event) => {
            if (!event.relatedTarget || !event.currentTarget.contains(event.relatedTarget as Node)) {
              setAddressFocused(false);
            }
          }}
        >
          <HardDrive size={15} className="sdkwork-sandbox-explorer__address-icon" />
          {addressFocused ? (
            <input
              ref={addressInputRef}
              className="sdkwork-sandbox-explorer__address-input"
              aria-label={labels.sandboxAbsolutePath}
              readOnly
              spellCheck={false}
              value={absolutePath}
              onClick={(event) => event.currentTarget.select()}
              onKeyDown={(event) => {
                if (event.key === 'Escape' || event.key === 'Enter') {
                  event.preventDefault();
                  event.currentTarget.blur();
                }
              }}
            />
          ) : (
            <div className="sdkwork-sandbox-explorer__address-breadcrumbs">
              {breadcrumbs.map((breadcrumb, index) => {
                const current = index === breadcrumbs.length - 1;
                return (
                  <span key={breadcrumb.logicalPath || 'root'} className="sdkwork-sandbox-explorer__crumb">
                    {index > 0 && <ChevronRight size={14} aria-hidden />}
                    <button
                      type="button"
                      aria-current={current ? 'page' : undefined}
                      disabled={loading}
                      onClick={(event) => {
                        event.stopPropagation();
                        if (current) {
                          setAddressFocused(true);
                        } else if (root) {
                          void loadDirectory(root, breadcrumb);
                        }
                      }}
                    >
                      {breadcrumb.label}
                    </button>
                  </span>
                );
              })}
            </div>
          )}
          <button
            type="button"
            className={`sdkwork-sandbox-explorer__address-copy${pathCopied ? ' is-copied' : ''}`}
            aria-label={pathCopied ? labels.pathCopied : labels.copyPath}
            title={pathCopied ? labels.pathCopied : labels.copyFullPath}
            disabled={!absolutePath}
            onClick={() => void copyCurrentPath()}
          >
            {pathCopied ? <Check size={14} /> : <Copy size={14} />}
          </button>
          <span className="sdkwork-sandbox-explorer__sr-only" aria-live="polite">
            {pathCopied ? labels.pathCopiedAnnouncement : ''}
          </span>
        </nav>

        <div className="sdkwork-sandbox-explorer__search">
          <Search size={14} aria-hidden />
          <label className="sdkwork-sandbox-explorer__sr-only" htmlFor={searchId}>
            {labels.filterLoadedItems}
          </label>
          <input
            ref={searchInputRef}
            id={searchId}
            type="search"
            value={searchQuery}
            placeholder={interpolateLabels(labels.searchPlaceholder, { name: currentLabel })}
            onChange={(event) => setSearchQuery(event.target.value)}
          />
          {searchQuery && (
            <button type="button" aria-label={labels.clearSearch} onClick={() => setSearchQuery('')}>
              <X size={13} />
            </button>
          )}
        </div>
      </div>

      <div className="sdkwork-sandbox-explorer__command-bar">
        {mode === 'manage' && (
          <button
            type="button"
            className="sdkwork-sandbox-explorer__command sdkwork-sandbox-explorer__command--primary"
            title={labels.newFolder}
            aria-label={labels.newFolder}
            disabled={!root?.capabilities.createDirectory || !directory || loading}
            onClick={() => {
              setNewDirectoryName('');
              setCreatingDirectory(true);
            }}
          >
            <FolderPlus size={16} />
            <span>{labels.newFolder}</span>
          </button>
        )}
        {mode === 'manage' && (
          <button
            type="button"
            className="sdkwork-sandbox-explorer__command sdkwork-sandbox-explorer__command--primary"
            title={labels.newFile}
            aria-label={labels.newFile}
            disabled={!root?.capabilities.createFile || !directory || loading}
            onClick={() => {
              setNewFileName('');
              setCreatingFile(true);
            }}
          >
            <FilePlus2 size={16} />
            <span>{labels.newFile}</span>
          </button>
        )}
        {mode === 'manage' && (
          <>
            <button
              type="button"
              className="sdkwork-sandbox-explorer__command sdkwork-sandbox-explorer__command--primary"
              title={labels.uploadFile}
              aria-label={labels.uploadFile}
              disabled={!root?.capabilities.createFile || !directory || loading || createPending}
              onClick={() => fileInputRef.current?.click()}
            >
              <Upload size={16} />
              <span>{uploadProgress
                ? interpolateLabels(labels.uploadingFiles, {
                  current: String(uploadProgress.current),
                  total: String(uploadProgress.total),
                })
                : labels.uploadFile}</span>
            </button>
            <input
              ref={fileInputRef}
              type="file"
              multiple
              hidden
              onChange={(event) => void handleUploadFiles(event.target.files)}
            />
          </>
        )}
        {mode === 'manage' && selectedEntry && (
          <>
            <span className="sdkwork-sandbox-explorer__separator" aria-hidden />
            <button
              type="button"
              className="sdkwork-sandbox-explorer__command"
              title={labels.rename}
              disabled={!root?.capabilities.moveEntry}
              onClick={() => startRename(selectedEntry)}
            >
              <Pencil size={15} />
              <span>{labels.rename}</span>
            </button>
            <button
              type="button"
              className="sdkwork-sandbox-explorer__command"
              title={labels.delete}
              disabled={!root?.capabilities.deleteEntry}
              onClick={() => setDeletingEntry(selectedEntry)}
            >
              <Trash2 size={15} />
              <span>{labels.delete}</span>
            </button>
          </>
        )}
        {mode === 'manage' && <span className="sdkwork-sandbox-explorer__separator" aria-hidden />}
        <button
          type="button"
          className="sdkwork-sandbox-explorer__command"
          title={sortAscending ? labels.sortDescending : labels.sortAscending}
          onClick={() => setSortAscending((current) => !current)}
        >
          <span>{labels.sort}</span>
          <ChevronDown size={13} />
        </button>
        <button
          type="button"
          className="sdkwork-sandbox-explorer__command"
          title={viewMode === 'details' ? labels.switchToGridView : labels.switchToDetailsView}
          onClick={() => setViewMode((current) => current === 'details' ? 'grid' : 'details')}
        >
          {viewMode === 'details' ? <List size={16} /> : <LayoutGrid size={16} />}
          <span>{labels.view}</span>
          <ChevronDown size={13} />
        </button>
        <div ref={moreMenuRef} className="sdkwork-sandbox-explorer__more-wrap">
          <button
            type="button"
            className="sdkwork-sandbox-explorer__command sdkwork-sandbox-explorer__command--more"
            title={labels.moreOptions}
            aria-label={labels.moreOptions}
            aria-haspopup="menu"
            aria-expanded={moreMenuOpen}
            onClick={() => setMoreMenuOpen((current) => !current)}
          >
            <MoreHorizontal size={18} />
          </button>
          {moreMenuOpen && (
            <div className="sdkwork-sandbox-explorer__more-menu" role="menu" aria-label={labels.moreOptions}>
              <button
                type="button"
                role="menuitem"
                disabled={!root || !directory || loading}
                onClick={() => {
                  setMoreMenuOpen(false);
                  if (root && directory) void loadDirectory(root, directory, 'preserve');
                }}
              >
                <RefreshCw size={15} />
                {labels.refresh}
                <kbd>F5</kbd>
              </button>
              {mode === 'manage' && (
                <>
                  <button
                    type="button"
                    role="menuitem"
                    disabled={!root?.capabilities.createDirectory || !directory || loading}
                    onClick={() => {
                      setMoreMenuOpen(false);
                      setNewDirectoryName('');
                      setCreatingDirectory(true);
                    }}
                  >
                    <FolderPlus size={15} />
                    {labels.newFolder}
                  </button>
                  <button
                    type="button"
                    role="menuitem"
                    disabled={!root?.capabilities.createFile || !directory || loading}
                    onClick={() => {
                      setMoreMenuOpen(false);
                      setNewFileName('');
                      setCreatingFile(true);
                    }}
                  >
                    <FilePlus2 size={15} />
                    {labels.newFile}
                  </button>
                </>
              )}
              <span className="sdkwork-sandbox-explorer__menu-separator" role="separator" />
              <button
                type="button"
                role="menuitem"
                onClick={() => {
                  setMoreMenuOpen(false);
                  setViewMode((current) => current === 'details' ? 'grid' : 'details');
                }}
              >
                {viewMode === 'details' ? <LayoutGrid size={15} /> : <List size={15} />}
                {viewMode === 'details' ? labels.gridView : labels.detailsView}
              </button>
              <button
                type="button"
                role="menuitem"
                onClick={() => {
                  setMoreMenuOpen(false);
                  setDetailsVisible((current) => !current);
                }}
              >
                <PanelRight size={15} />
                {detailsVisible ? labels.hideDetailsPane : labels.showDetailsPane}
              </button>
            </div>
          )}
        </div>
        <button
          type="button"
          className={`sdkwork-sandbox-explorer__command sdkwork-sandbox-explorer__command--details${detailsVisible ? ' is-active' : ''}`}
          title={detailsVisible ? labels.hideDetailsPane : labels.showDetailsPane}
          aria-label={detailsVisible ? labels.hideDetailsPane : labels.showDetailsPane}
          aria-pressed={detailsVisible}
          onClick={() => setDetailsVisible((current) => !current)}
        >
          <PanelRight size={16} />
          <span>{labels.details}</span>
        </button>
      </div>

      {error && (
        <div role="alert" className="sdkwork-sandbox-explorer__alert">
          <Info size={15} />
          <span>{error}</span>
          {root && directory && (
            <button
              type="button"
              className="sdkwork-sandbox-explorer__alert-retry"
              disabled={loading || refreshing}
              onClick={() => void loadDirectory(root, directory, 'preserve')}
            >
              {labels.reload}
            </button>
          )}
          {!root && (
            <button
              type="button"
              className="sdkwork-sandbox-explorer__alert-retry"
              disabled={loading}
              onClick={() => setSandboxLoadAttempt((current) => current + 1)}
            >
              {labels.retry}
            </button>
          )}
          <button type="button" title={labels.dismiss} aria-label={labels.dismiss} onClick={() => setError(null)}>
            <X size={14} />
          </button>
        </div>
      )}

      <div className={`sdkwork-sandbox-explorer__workspace${detailsVisible ? '' : ' is-details-hidden'}`}>
        <aside className="sdkwork-sandbox-explorer__sidebar" aria-label={labels.sandboxNavigation}>
          <div className="sdkwork-sandbox-explorer__sidebar-heading">
            <ChevronDown size={13} aria-hidden />
            <Server size={15} aria-hidden />
            <span>{labels.sandboxes}</span>
          </div>
          <label className="sdkwork-sandbox-explorer__sr-only" htmlFor={sandboxSelectId}>{labels.sandbox}</label>
          <select
            id={sandboxSelectId}
            value={root?.id ?? ''}
            disabled={roots.length === 0 || loading}
            onChange={(event) => {
              const nextRoot = roots.find((candidate) => candidate.id === event.target.value);
              if (nextRoot) {
                void loadDirectory(nextRoot, {
                  entryId: nextRoot.rootEntryId,
                  logicalPath: '',
                });
              }
            }}
          >
            {roots.map((candidate) => (
              <option key={candidate.id} value={candidate.id}>{candidate.displayName}</option>
            ))}
          </select>

          <div className="sdkwork-sandbox-explorer__tree" role="tree" aria-label={labels.availableSandboxes}>
            {roots.map((candidate) => {
              const active = candidate.id === root?.id;
              return (
                <div key={candidate.id} className="sdkwork-sandbox-explorer__tree-group">
                  <button
                    type="button"
                    role="treeitem"
                    aria-label={interpolateLabels(labels.openSandbox, { name: candidate.displayName })}
                    aria-selected={active}
                    className={`sdkwork-sandbox-explorer__tree-item${active ? ' is-active' : ''}`}
                    disabled={loading && !active}
                    onClick={() => {
                      if (!active) {
                        void loadDirectory(candidate, {
                          entryId: candidate.rootEntryId,
                          logicalPath: '',
                        });
                      }
                    }}
                  >
                    {active ? <ChevronDown size={13} /> : <ChevronRight size={13} />}
                    <HardDrive size={15} />
                    <span>{candidate.displayName}</span>
                  </button>
                  {active && breadcrumbs.slice(1).map((breadcrumb, index) => (
                    <button
                      key={breadcrumb.logicalPath}
                      type="button"
                      role="treeitem"
                      aria-current={index === breadcrumbs.length - 2 ? 'page' : undefined}
                      className="sdkwork-sandbox-explorer__tree-child"
                      disabled={loading || index === breadcrumbs.length - 2}
                      onClick={() => void loadDirectory(candidate, breadcrumb)}
                    >
                      <FolderOpen size={14} />
                      <span>{breadcrumb.label}</span>
                    </button>
                  ))}
                </div>
              );
            })}
          </div>

          {sandboxPage < sandboxTotalPages && (
            <button
              type="button"
              className="sdkwork-sandbox-explorer__load-roots"
              disabled={loadingMoreSandboxes}
              onClick={() => void loadMoreSandboxes()}
            >
              {loadingMoreSandboxes ? <LoaderCircle size={14} className="is-spinning" /> : <MoreHorizontal size={15} />}
              <span>{labels.moreSandboxes}</span>
            </button>
          )}
        </aside>

        <main
          ref={contentRef}
          className="sdkwork-sandbox-explorer__content"
          aria-busy={loading || refreshing}
          tabIndex={-1}
          onContextMenu={(event) => {
            if ((event.target as HTMLElement).closest('[data-entry-id], input, button, textarea, select')) return;
            event.preventDefault();
            openBackgroundContextMenu(
              event.clientX,
              event.clientY,
              document.activeElement instanceof HTMLElement ? document.activeElement : contentRef.current,
            );
          }}
        >
          <div className="sdkwork-sandbox-explorer__content-heading">
            <ChevronDown size={13} aria-hidden />
            <span>{currentLabel}</span>
          </div>

          {creatingDirectory && root?.capabilities.createDirectory && directory && (
            <form className="sdkwork-sandbox-explorer__create-row" onSubmit={(event) => void submitCreateDirectory(event)}>
              <Folder size={18} />
              <label className="sdkwork-sandbox-explorer__sr-only" htmlFor={newDirectoryNameId}>{labels.folderName}</label>
              <input
                id={newDirectoryNameId}
                autoFocus
                required
                maxLength={255}
                value={newDirectoryName}
                placeholder={labels.folderName}
                disabled={createPending}
                onChange={(event) => setNewDirectoryName(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === 'Escape') {
                    setCreatingDirectory(false);
                    setNewDirectoryName('');
                  }
                }}
              />
              <button type="submit" title={labels.createFolder} aria-label={labels.createFolder} disabled={!newDirectoryName.trim() || createPending}>
                {createPending ? <LoaderCircle size={15} className="is-spinning" /> : <Check size={15} />}
              </button>
              <button
                type="button"
                title={labels.cancel}
                aria-label={labels.cancel}
                disabled={createPending}
                onClick={() => {
                  setCreatingDirectory(false);
                  setNewDirectoryName('');
                }}
              >
                <X size={15} />
              </button>
            </form>
          )}

          {creatingFile && root?.capabilities.createFile && directory && (
            <form className="sdkwork-sandbox-explorer__create-row" onSubmit={(event) => void submitCreateFile(event)}>
              <FilePlus2 size={18} />
              <label className="sdkwork-sandbox-explorer__sr-only" htmlFor={`${newDirectoryNameId}-file`}>{labels.fileName}</label>
              <input
                id={`${newDirectoryNameId}-file`}
                autoFocus
                required
                maxLength={255}
                value={newFileName}
                placeholder={labels.fileName}
                disabled={createPending}
                onChange={(event) => setNewFileName(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === 'Escape') {
                    setCreatingFile(false);
                    setNewFileName('');
                  }
                }}
              />
              <button type="submit" title={labels.createFile} aria-label={labels.createFile} disabled={!newFileName.trim() || createPending}>
                {createPending ? <LoaderCircle size={15} className="is-spinning" /> : <Check size={15} />}
              </button>
              <button
                type="button"
                title={labels.cancelFileCreation}
                aria-label={labels.cancelFileCreation}
                disabled={createPending}
                onClick={() => {
                  setCreatingFile(false);
                  setNewFileName('');
                }}
              >
                <X size={15} />
              </button>
            </form>
          )}

          {loading ? (
            <div className="sdkwork-sandbox-explorer__state">
              <LoaderCircle size={22} className="is-spinning" aria-label={labels.loading} />
              <span>{labels.loadingFolder}</span>
            </div>
          ) : roots.length === 0 ? (
            <div className="sdkwork-sandbox-explorer__state">
              <Server size={34} />
              <span>{labels.noAccessibleSandboxes}</span>
            </div>
          ) : visibleEntries.length === 0 ? (
            <div className="sdkwork-sandbox-explorer__state">
              <FolderOpen size={34} />
              <span>{searchQuery ? labels.noSearchMatches : labels.folderEmpty}</span>
            </div>
          ) : viewMode === 'details' ? (
            <div className="sdkwork-sandbox-explorer__details-view" aria-label={labels.directoryItems}>
              <div className="sdkwork-sandbox-explorer__columns" aria-hidden>
                <span>{labels.nameColumn}</span>
                <span>{labels.typeColumn}</span>
                <span>{labels.locationColumn}</span>
              </div>
              {visibleEntries.map((entry) => (
                <button
                  key={entry.id}
                  ref={(element) => {
                    if (element) entryElementsById.current.set(entry.id, element);
                    else entryElementsById.current.delete(entry.id);
                  }}
                  data-entry-id={entry.id}
                  type="button"
                  aria-label={entry.name}
                  tabIndex={rovingEntryId === entry.id ? 0 : -1}
                  className={`sdkwork-sandbox-explorer__entry-row${selectedEntry?.id === entry.id ? ' is-selected' : ''}`}
                  onFocus={() => setSelectedEntry(entry)}
                  onClick={() => selectEntry(entry)}
                  onDoubleClick={() => activateEntry(entry)}
                  onContextMenu={(event) => {
                    event.preventDefault();
                    event.stopPropagation();
                    openEntryContextMenu(entry, event.clientX, event.clientY, event.currentTarget);
                  }}
                >
                  <span className="sdkwork-sandbox-explorer__entry-name">
                    {entry.kind === 'directory'
                      ? <Folder size={18} className="is-folder" />
                      : <File size={18} className="is-file" />}
                    <span title={entry.name}>{entry.name}</span>
                  </span>
                  <span>{entryType(entry, labels)}</span>
                  <span title={entryLocation(entry)}>{entryLocation(entry)}</span>
                </button>
              ))}
            </div>
          ) : (
            <div className="sdkwork-sandbox-explorer__grid-view" aria-label={labels.directoryItems}>
              {visibleEntries.map((entry) => (
                <button
                  key={entry.id}
                  ref={(element) => {
                    if (element) entryElementsById.current.set(entry.id, element);
                    else entryElementsById.current.delete(entry.id);
                  }}
                  data-entry-id={entry.id}
                  type="button"
                  aria-label={entry.name}
                  tabIndex={rovingEntryId === entry.id ? 0 : -1}
                  className={`sdkwork-sandbox-explorer__entry-card${selectedEntry?.id === entry.id ? ' is-selected' : ''}`}
                  onFocus={() => setSelectedEntry(entry)}
                  onClick={() => selectEntry(entry)}
                  onDoubleClick={() => activateEntry(entry)}
                  onContextMenu={(event) => {
                    event.preventDefault();
                    event.stopPropagation();
                    openEntryContextMenu(entry, event.clientX, event.clientY, event.currentTarget);
                  }}
                >
                  {entry.kind === 'directory'
                    ? <Folder size={42} className="is-folder" />
                    : <File size={42} className="is-file" />}
                  <span title={entry.name}>{entry.name}</span>
                </button>
              ))}
            </div>
          )}

          {nextCursor && !loading && (
            <button
              ref={loadMoreRef}
              type="button"
              aria-label={labels.loadMore}
              className="sdkwork-sandbox-explorer__load-more"
              disabled={loadingMore}
              onClick={() => void loadMoreEntries()}
            >
              {loadingMore && <LoaderCircle size={14} className="is-spinning" />}
              {loadingMore ? labels.loadingMoreItems : labels.loadMoreItems}
            </button>
          )}
        </main>

        {detailsVisible && (
          <aside className="sdkwork-sandbox-explorer__details-pane" aria-label={labels.itemDetails}>
            <div className="sdkwork-sandbox-explorer__preview-icon">
              {selectedEntry?.kind === 'file'
                ? <File size={64} className="is-file" />
                : <FolderOpen size={64} className="is-folder" />}
            </div>
            <div className="sdkwork-sandbox-explorer__detail-copy">
              <h2 title={detailName}>{detailName}</h2>
              <dl>
                <div>
                  <dt>{labels.type}</dt>
                  <dd>{detailKind}</dd>
                </div>
                <div>
                  <dt>{labels.sandbox}</dt>
                  <dd>{root?.displayName ?? '—'}</dd>
                </div>
                <div>
                  <dt>{labels.location}</dt>
                  <dd>{selectedEntry ? entryLocation(selectedEntry) : directory?.logicalPath || '/'}</dd>
                </div>
              </dl>
              {mode === 'manage' && selectedEntry && (
                <div className="sdkwork-sandbox-explorer__detail-actions">
                  <button type="button" onClick={() => activateEntry(selectedEntry)}>
                    {selectedEntry.kind === 'directory'
                      ? <FolderOpen size={14} />
                      : <FilePenLine size={14} />}
                    {selectedEntry.kind === 'directory' ? labels.open : labels.openFile}
                  </button>
                  <button
                    type="button"
                    disabled={!root?.capabilities.moveEntry}
                    onClick={() => startRename(selectedEntry)}
                  >
                    <Pencil size={14} />
                    {labels.rename}
                  </button>
                </div>
              )}
              {!selectedEntry && (
                <div className="sdkwork-sandbox-explorer__detail-hint">
                  <Info size={15} />
                  <span>{labels.selectItemDetails}</span>
                </div>
              )}
            </div>
          </aside>
        )}
      </div>

      <footer className="sdkwork-sandbox-explorer__status-bar">
        <span>{interpolateLabels(visibleEntries.length === 1 ? labels.itemsLoadedOne : labels.itemsLoadedMany, { count: String(visibleEntries.length) })}</span>
        {searchQuery && <span>{interpolateLabels(labels.filteredFrom, { count: String(entries.length) })}</span>}
        {filtering && <span role="status">{labels.filtering}</span>}
        {refreshing && <span role="status">{labels.refreshing}</span>}
        {!searchQuery && entries.length > 0 && (
          <span role="status" aria-live="polite">
            {loadingMore ? labels.loadingMoreItems : nextCursor ? labels.moreItemsAvailable : labels.allItemsLoaded}
          </span>
        )}
        <span className="sdkwork-sandbox-explorer__status-path">
          {root && directory ? currentSelection(root, directory).displayPath : labels.noSandboxSelected}
        </span>
        {mode === 'select-directory' && root && directory && (
          <button
            type="button"
            className="sdkwork-sandbox-explorer__select-button"
            disabled={!root.capabilities.selectDirectory || loading}
            onClick={selectCurrentDirectory}
          >
            <FolderOpen size={15} />
            {labels.selectDirectory}
          </button>
        )}
      </footer>

      {contextMenu && (
        <div
          ref={contextMenuRef}
          className={`sdkwork-sandbox-explorer__context-menu sdkwork-sandbox-explorer__context-menu--${platform}`}
          role="menu"
          aria-label={contextMenu.kind === 'entry'
            ? interpolateLabels(labels.entryActions, { name: contextMenu.entry?.name ?? '' })
            : labels.currentFolderActions}
          style={{ left: contextMenu.x, top: contextMenu.y }}
          onKeyDown={handleContextMenuKeyDown}
        >
          {contextMenu.kind === 'entry' && contextMenu.entry ? (
            <>
              <button type="button" role="menuitem" className="is-default" onClick={() => activateEntry(contextMenu.entry!)}>
                {contextMenu.entry.kind === 'directory' ? <FolderOpen size={15} /> : <FilePenLine size={15} />}
                <span>{labels.open}</span>
                <kbd>{platform === 'macos' ? '⌘O' : 'Enter'}</kbd>
              </button>
              <span role="separator" />
              <button
                type="button"
                role="menuitem"
                onClick={() => {
                  const path = entryAbsolutePath(contextMenu.entry!);
                  setContextMenu(null);
                  void copyPath(path);
                }}
              >
                <Copy size={15} />
                <span>{platform === 'windows' ? labels.copyAsPath : platform === 'macos' ? labels.copyPathname : labels.copyLocation}</span>
                <kbd>{platform === 'macos' ? '⌥⌘C' : 'Ctrl+Shift+C'}</kbd>
              </button>
              {mode === 'manage' && (
                <>
                  <button
                    type="button"
                    role="menuitem"
                    disabled={!root?.capabilities.moveEntry}
                    onClick={() => startRename(contextMenu.entry!)}
                  >
                    <Pencil size={15} />
                    <span>{labels.rename}</span>
                    <kbd>{platform === 'macos' ? 'Return' : 'F2'}</kbd>
                  </button>
                  <button
                    type="button"
                    role="menuitem"
                    disabled={!root?.capabilities.moveEntry}
                    onClick={() => startMove(contextMenu.entry!)}
                  >
                    <Move size={15} />
                    <span>{labels.moveTo}</span>
                    <kbd />
                  </button>
                </>
              )}
              <span role="separator" />
              <button
                type="button"
                role="menuitem"
                onClick={() => {
                  setPropertiesTarget({ kind: 'entry', entry: contextMenu.entry });
                  setContextMenu(null);
                }}
              >
                <Info size={15} />
                <span>{platform === 'macos' ? labels.getInfo : labels.properties}</span>
                <kbd>{platform === 'macos' ? '⌘I' : 'Alt+Enter'}</kbd>
              </button>
              {mode === 'manage' && (
                <button
                  type="button"
                  role="menuitem"
                  className="is-danger"
                  disabled={!root?.capabilities.deleteEntry}
                  onClick={() => {
                    setContextMenu(null);
                    setDeletingEntry(contextMenu.entry!);
                  }}
                >
                  <Trash2 size={15} />
                  <span>{labels.deletePermanently}…</span>
                  <kbd>{platform === 'macos' ? '⌥⌘⌫' : 'Shift+Del'}</kbd>
                </button>
              )}
            </>
          ) : (
            <>
              {mode === 'manage' && (
                <>
                  <button
                    type="button"
                    role="menuitem"
                    disabled={!root?.capabilities.createDirectory || !directory || loading}
                    onClick={() => {
                      setContextMenu(null);
                      setNewDirectoryName('');
                      setCreatingDirectory(true);
                    }}
                  >
                    <FolderPlus size={15} />
                    <span>{labels.newFolder}</span>
                    <kbd>{platform === 'macos' ? '⇧⌘N' : 'Ctrl+Shift+N'}</kbd>
                  </button>
                  <button
                    type="button"
                    role="menuitem"
                    disabled={!root?.capabilities.createFile || !directory || loading}
                    onClick={() => {
                      setContextMenu(null);
                      setNewFileName('');
                      setCreatingFile(true);
                    }}
                  >
                    <FilePlus2 size={15} />
                    <span>{labels.newFile}</span>
                    <kbd />
                  </button>
                  <span role="separator" />
                </>
              )}
              <button
                type="button"
                role="menuitem"
                disabled={!root || !directory || loading}
                onClick={() => {
                  setContextMenu(null);
                  if (root && directory) void loadDirectory(root, directory, 'preserve');
                }}
              >
                <RefreshCw size={15} />
                <span>{labels.refresh}</span>
                <kbd>{platform === 'macos' ? '⌘R' : 'F5'}</kbd>
              </button>
              <button
                type="button"
                role="menuitemcheckbox"
                aria-checked={sortAscending}
                onClick={() => {
                  setSortAscending((current) => !current);
                  setContextMenu(null);
                }}
              >
                <ArrowDown size={15} className={sortAscending ? 'is-ascending' : 'is-descending'} />
                <span>{sortAscending ? labels.sortDescending : labels.sortAscending}</span>
                <kbd />
              </button>
              <button
                type="button"
                role="menuitemradio"
                aria-checked={viewMode === 'details'}
                onClick={() => {
                  setViewMode((current) => current === 'details' ? 'grid' : 'details');
                  setContextMenu(null);
                }}
              >
                {viewMode === 'details' ? <LayoutGrid size={15} /> : <List size={15} />}
                <span>{viewMode === 'details' ? labels.gridView : labels.detailsView}</span>
                <kbd />
              </button>
              <button
                type="button"
                role="menuitemcheckbox"
                aria-checked={detailsVisible}
                onClick={() => {
                  setDetailsVisible((current) => !current);
                  setContextMenu(null);
                }}
              >
                <PanelRight size={15} />
                <span>{detailsVisible ? labels.hideDetailsPane : labels.showDetailsPane}</span>
                <kbd />
              </button>
              <span role="separator" />
              <button
                type="button"
                role="menuitem"
                disabled={!absolutePath}
                onClick={() => {
                  setContextMenu(null);
                  void copyPath(absolutePath);
                }}
              >
                <Copy size={15} />
                <span>{platform === 'windows' ? labels.copyCurrentPath : platform === 'macos' ? labels.copyPathname : labels.copyLocation}</span>
                <kbd />
              </button>
              <button
                type="button"
                role="menuitem"
                disabled={!root || !directory}
                onClick={() => {
                  setPropertiesTarget({ kind: 'directory' });
                  setContextMenu(null);
                }}
              >
                <Info size={15} />
                <span>{platform === 'macos' ? labels.getInfo : labels.properties}</span>
                <kbd>{platform === 'macos' ? '⌘I' : 'Alt+Enter'}</kbd>
              </button>
            </>
          )}
        </div>
      )}

      {fileEditor && (
        <div className="sdkwork-sandbox-explorer__modal-backdrop" role="presentation">
          <section className="sdkwork-sandbox-explorer__operation-dialog sdkwork-sandbox-explorer__editor" role="dialog" aria-modal="true" aria-label={interpolateLabels(labels.editFile, { name: fileEditor.entry.name })}>
            <header>
              <FilePenLine size={16} />
              <strong>{fileEditor.entry.name}</strong>
              <span>{fileEditor.encoding === 'utf8'
                ? interpolateLabels(labels.bytes, { count: fileEditor.sizeBytes })
                : labels.binaryPreview}</span>
              <button type="button" aria-label={labels.closeFile} onClick={() => setFileEditor(null)}><X size={16} /></button>
            </header>
            {fileEditor.loading ? (
              <div className="sdkwork-sandbox-explorer__operation-state"><LoaderCircle size={20} className="is-spinning" />{labels.loadingFile}</div>
            ) : (
              <>
                {fileEditor.error && <div role="alert" className="sdkwork-sandbox-explorer__editor-error">{fileEditor.error}</div>}
                <textarea
                  aria-label={labels.fileContent}
                  readOnly={fileEditor.encoding === 'base64' || !root?.capabilities.writeFile}
                  value={fileEditor.content}
                  spellCheck={false}
                  onChange={(event) => setFileEditor((current) => current ? { ...current, content: event.target.value } : null)}
                />
                <footer>
                  <span>{fileEditor.encoding === 'base64'
                    ? labels.base64ReadOnly
                    : fileEditor.checksumSha256
                      ? interpolateLabels(labels.sha256, { digest: `${fileEditor.checksumSha256.slice(0, 12)}…` })
                      : labels.utf8Text}</span>
                  <button type="button" onClick={() => setFileEditor(null)}>{labels.close}</button>
                  {fileEditor.encoding === 'utf8' && root?.capabilities.writeFile && (
                    <button type="button" className="is-primary" disabled={fileEditor.saving} onClick={() => void saveFile()}>
                      {fileEditor.saving ? <LoaderCircle size={14} className="is-spinning" /> : <Save size={14} />}
                      {labels.save}
                    </button>
                  )}
                </footer>
              </>
            )}
          </section>
        </div>
      )}

      {propertiesTarget && root && directory && (
        <OperationDialog
          title={platform === 'macos' ? labels.info : labels.properties}
          closeLabel={labels.close}
          onCancel={() => setPropertiesTarget(null)}
        >
          <div className="sdkwork-sandbox-explorer__properties">
            <div className="sdkwork-sandbox-explorer__properties-icon">
              {propertiesTarget.entry?.kind === 'file'
                ? <File size={42} className="is-file" />
                : <FolderOpen size={42} className="is-folder" />}
            </div>
            <dl>
              <div><dt>{labels.name}</dt><dd>{propertiesTarget.entry?.name ?? currentLabel}</dd></div>
              <div><dt>{labels.kind}</dt><dd>{propertiesTarget.entry ? entryType(propertiesTarget.entry, labels) : labels.sandboxFolderType}</dd></div>
              <div><dt>{labels.sandbox}</dt><dd>{root.displayName}</dd></div>
              <div><dt>{labels.location}</dt><dd>{propertiesTarget.entry ? entryLocation(propertiesTarget.entry) : directory.logicalPath || '/'}</dd></div>
              <div><dt>{labels.logicalPath}</dt><dd>{propertiesTarget.entry ? entryAbsolutePath(propertiesTarget.entry) : absolutePath}</dd></div>
              {propertiesTarget.entry && <div><dt>{labels.revision}</dt><dd>{propertiesTarget.entry.revision}</dd></div>}
            </dl>
            <div className="sdkwork-sandbox-explorer__dialog-actions">
              <button
                type="button"
                onClick={() => void copyPath(propertiesTarget.entry ? entryAbsolutePath(propertiesTarget.entry) : absolutePath)}
              >
                <Copy size={14} />
                {labels.copyPath}
              </button>
              <button type="button" className="is-primary" onClick={() => setPropertiesTarget(null)}>{labels.ok}</button>
            </div>
          </div>
        </OperationDialog>
      )}

      {renamingEntry && (
        <OperationDialog title={interpolateLabels(labels.renameDialogTitle, { name: renamingEntry.name })} closeLabel={labels.close} onCancel={() => setRenamingEntry(null)}>
          <form onSubmit={(event) => void submitRename(event)}>
            <label htmlFor={`${newDirectoryNameId}-rename`}>{labels.newName}</label>
            <input id={`${newDirectoryNameId}-rename`} autoFocus required maxLength={255} value={renameValue} disabled={mutationPending} onChange={(event) => setRenameValue(event.target.value)} />
            <DialogActions pending={mutationPending} submitLabel={labels.rename} cancelLabel={labels.cancel} onCancel={() => setRenamingEntry(null)} />
          </form>
        </OperationDialog>
      )}

      {movingEntry && (
        <OperationDialog title={interpolateLabels(labels.moveDialogTitle, { name: movingEntry.name })} closeLabel={labels.close} onCancel={() => setMovingEntry(null)}>
          <form onSubmit={(event) => void submitMove(event)}>
            <label htmlFor={`${newDirectoryNameId}-move`}>{labels.destinationFolderPath}</label>
            <input id={`${newDirectoryNameId}-move`} autoFocus placeholder={labels.emptyForSandboxRoot} value={moveDestination} disabled={mutationPending} onChange={(event) => setMoveDestination(event.target.value)} />
            <p>{labels.moveHint}</p>
            <DialogActions pending={mutationPending} submitLabel={labels.move} cancelLabel={labels.cancel} onCancel={() => setMovingEntry(null)} />
          </form>
        </OperationDialog>
      )}

      {deletingEntry && (
        <OperationDialog title={interpolateLabels(labels.deleteDialogTitle, { name: deletingEntry.name })} closeLabel={labels.close} onCancel={() => setDeletingEntry(null)} danger>
          <p>{interpolateLabels(labels.deleteConfirm, {
            kind: deletingEntry.kind === 'directory' ? labels.directoryKindNoun : labels.fileKindNoun,
          })}</p>
          <div className="sdkwork-sandbox-explorer__dialog-actions">
            <button type="button" onClick={() => setDeletingEntry(null)} disabled={mutationPending}>{labels.cancel}</button>
            <button type="button" className="is-danger" onClick={() => void confirmDelete()} disabled={mutationPending}>
              {mutationPending && <LoaderCircle size={14} className="is-spinning" />}
              {labels.deletePermanently}
            </button>
          </div>
        </OperationDialog>
      )}
    </section>
  );
}

function OperationDialog({
  title,
  children,
  onCancel,
  danger = false,
  closeLabel,
}: {
  readonly title: string;
  readonly children: React.ReactNode;
  readonly onCancel: () => void;
  readonly danger?: boolean;
  readonly closeLabel: string;
}) {
  return (
    <div className="sdkwork-sandbox-explorer__modal-backdrop" role="presentation" onMouseDown={(event) => event.target === event.currentTarget && onCancel()}>
      <section className={`sdkwork-sandbox-explorer__operation-dialog${danger ? ' is-danger' : ''}`} role="dialog" aria-modal="true" aria-label={title}>
        <header><strong>{title}</strong><button type="button" aria-label={closeLabel} onClick={onCancel}><X size={16} /></button></header>
        <div className="sdkwork-sandbox-explorer__operation-body">{children}</div>
      </section>
    </div>
  );
}

function DialogActions({
  pending,
  submitLabel,
  onCancel,
  cancelLabel,
}: {
  readonly pending: boolean;
  readonly submitLabel: string;
  readonly onCancel: () => void;
  readonly cancelLabel: string;
}) {
  return (
    <div className="sdkwork-sandbox-explorer__dialog-actions">
      <button type="button" onClick={onCancel} disabled={pending}>{cancelLabel}</button>
      <button type="submit" className="is-primary" disabled={pending}>
        {pending && <LoaderCircle size={14} className="is-spinning" />}
        {submitLabel}
      </button>
    </div>
  );
}
