/**
 * Electron secure store.
 *
 * Tokens and secrets use `safeStorage` (OS-backed keychain: DPAPI on Windows,
 * Keychain on macOS, libsecret on Linux) and are persisted inside the app's
 * user-private directory, never in the generated renderer tree.
 *
 * The store is exposed only through the host adapter contract; the renderer
 * never reaches `safeStorage` directly.
 */

import { app, safeStorage } from 'electron';
import fs from 'node:fs';
import path from 'node:path';
import { BridgeError } from '../shared/ipc-channels';

export interface SecureStore {
  get(key: string): string | null;
  set(key: string, value: string): void;
  remove(key: string): void;
  clear(): void;
}

const STORE_FILE_NAME = 'sdkwork-drive-secure-store.bin';

function resolveStorePath(): string {
  return path.join(app.getPath('userData'), STORE_FILE_NAME);
}

function readEntries(): Record<string, string> {
  const storePath = resolveStorePath();
  if (!fs.existsSync(storePath)) {
    return {};
  }
  try {
    const raw = fs.readFileSync(storePath);
    const decrypted = safeStorage.isEncryptionAvailable()
      ? safeStorage.decryptString(raw)
      : raw.toString('utf8');
    return JSON.parse(decrypted) as Record<string, string>;
  } catch {
    // A corrupt or key-rotated store must degrade to empty rather than block
    // application boot.
    return {};
  }
}

function writeEntries(entries: Record<string, string>): void {
  const serialized = JSON.stringify(entries);
  const payload = safeStorage.isEncryptionAvailable()
    ? safeStorage.encryptString(serialized)
    : Buffer.from(serialized, 'utf8');
  fs.writeFileSync(resolveStorePath(), payload, { mode: 0o600 });
}

export function createSecureStore(): SecureStore {
  return {
    get(key) {
      return readEntries()[key] ?? null;
    },
    set(key, value) {
      if (!safeStorage.isEncryptionAvailable()) {
        throw new BridgeError(
          'unavailable',
          'OS-backed secure storage is not available on this platform.',
        );
      }
      const entries = readEntries();
      entries[key] = value;
      writeEntries(entries);
    },
    remove(key) {
      const entries = readEntries();
      delete entries[key];
      writeEntries(entries);
    },
    clear() {
      const storePath = resolveStorePath();
      if (fs.existsSync(storePath)) {
        fs.rmSync(storePath, { force: true });
      }
    },
  };
}
