import type { LocalFilesystemEntry } from '../types';
import type { NativeLocalUploadDescriptor } from './nativeLocalUploadFile';

export type WindowControlAction = 'minimize' | 'maximize' | 'unmaximize' | 'close' | 'show';

export interface HostAdapter {
  isNativeHost: boolean;
  windowControl(action: WindowControlAction): Promise<void>;
  openExternal(url: string): Promise<void>;
  writeTextToClipboard(text: string): Promise<void>;
  listLocalFilesystem(path?: string | null): Promise<LocalFilesystemEntry[]>;
  openLocalPath(path: string): Promise<void>;
  pickLocalUploadFiles(): Promise<NativeLocalUploadDescriptor[]>;
  describeLocalUploadFile(path: string): Promise<NativeLocalUploadDescriptor>;
  readLocalUploadRange(path: string, offsetBytes: number, lengthBytes: number): Promise<ArrayBuffer>;
  checksumLocalUploadFile(path: string): Promise<string>;
  saveDownloadFile(fileName: string, blob: Blob): Promise<boolean>;
  beginDownloadSave(fileName: string): Promise<string | null>;
  writeDownloadChunk(sessionId: string, chunk: Uint8Array): Promise<void>;
  finishDownloadSave(sessionId: string): Promise<boolean>;
  abortDownloadSave(sessionId: string): Promise<void>;
}

interface TauriGlobal {
  core?: {
    invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
  };
  shell?: {
    open(url: string): Promise<void>;
  };
  clipboard?: {
    writeText(text: string): Promise<void>;
  };
}

function getTauriGlobal(): TauriGlobal | undefined {
  return (globalThis as typeof globalThis & { __TAURI__?: TauriGlobal }).__TAURI__;
}

function assertSafeExternalUrl(url: string): void {
  const parsed = new URL(url);
  if (parsed.protocol !== 'https:' && parsed.protocol !== 'http:') {
    throw new Error('Only HTTP(S) URLs can be opened externally.');
  }
}

function unsupportedNativeUploadOperation(operation: string): never {
  throw new Error(`Native upload ${operation} is only available in the desktop app.`);
}

function unsupportedNativeDownloadOperation(operation: string): never {
  throw new Error(`Native download ${operation} is only available in the desktop app.`);
}

export function createHostAdapter(): HostAdapter {
  return {
    get isNativeHost() {
      return Boolean(getTauriGlobal());
    },
    async windowControl(action) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        return;
      }
      await tauri.core.invoke('window_control', { request: { action } });
    },
    async openExternal(url) {
      assertSafeExternalUrl(url);
      const tauri = getTauriGlobal();
      if (tauri?.shell?.open) {
        await tauri.shell.open(url);
        return;
      }
      globalThis.open?.(url, '_blank', 'noopener,noreferrer');
    },
    async writeTextToClipboard(text) {
      const tauri = getTauriGlobal();
      if (tauri?.clipboard?.writeText) {
        await tauri.clipboard.writeText(text);
        return;
      }
      await navigator.clipboard?.writeText(text);
    },
    async listLocalFilesystem(path) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        return [];
      }
      return tauri.core.invoke<LocalFilesystemEntry[]>('local_filesystem_list', {
        request: {
          path: path ?? null,
        },
      });
    },
    async openLocalPath(path) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        throw new Error('Local filesystem access is only available in the desktop app.');
      }
      await tauri.core.invoke('local_filesystem_open', {
        request: { path },
      });
    },
    async pickLocalUploadFiles() {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        return [];
      }
      return tauri.core.invoke<NativeLocalUploadDescriptor[]>('local_upload_pick_files');
    },
    async describeLocalUploadFile(path) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        unsupportedNativeUploadOperation('describe');
      }
      return tauri.core.invoke<NativeLocalUploadDescriptor>('local_upload_describe_file', {
        request: { path },
      });
    },
    async readLocalUploadRange(path, offsetBytes, lengthBytes) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        unsupportedNativeUploadOperation('read');
      }
      const response = await tauri.core.invoke<{ bytes: number[] }>('local_upload_read_range', {
        request: {
          path,
          offsetBytes,
          lengthBytes,
        },
      });
      return Uint8Array.from(response.bytes).buffer;
    },
    async checksumLocalUploadFile(path) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        unsupportedNativeUploadOperation('checksum');
      }
      const response = await tauri.core.invoke<{ checksumSha256Hex: string }>('local_upload_checksum_file', {
        request: { path },
      });
      return response.checksumSha256Hex;
    },
    async saveDownloadFile(fileName, blob) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        throw new Error('Native download save is only available in the desktop app.');
      }
      const bytes = Array.from(new Uint8Array(await blob.arrayBuffer()));
      const response = await tauri.core.invoke<{ saved: boolean }>('local_download_save', {
        request: {
          fileName,
          bytes,
        },
      });
      return response.saved;
    },
    async beginDownloadSave(fileName) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        unsupportedNativeDownloadOperation('begin');
      }
      const response = await tauri.core.invoke<{ sessionId: string; saved: boolean }>('local_download_begin', {
        request: { fileName },
      });
      return response.saved ? response.sessionId : null;
    },
    async writeDownloadChunk(sessionId, chunk) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        unsupportedNativeDownloadOperation('write');
      }
      await tauri.core.invoke('local_download_write_chunk', {
        request: {
          sessionId,
          bytes: Array.from(chunk),
        },
      });
    },
    async finishDownloadSave(sessionId) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        unsupportedNativeDownloadOperation('finish');
      }
      const response = await tauri.core.invoke<{ saved: boolean }>('local_download_finish', {
        request: { sessionId },
      });
      return response.saved;
    },
    async abortDownloadSave(sessionId) {
      const tauri = getTauriGlobal();
      if (!tauri?.core?.invoke) {
        unsupportedNativeDownloadOperation('abort');
      }
      await tauri.core.invoke('local_download_abort', {
        request: { sessionId },
      });
    },
  };
}
