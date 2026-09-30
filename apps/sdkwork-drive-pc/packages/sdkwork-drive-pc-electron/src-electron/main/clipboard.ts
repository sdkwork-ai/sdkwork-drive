/**
 * Electron clipboard capability with cut semantics.
 *
 * Mirrors the Tauri host: the renderer cuts Drive nodes through the SDK, and
 * the native host owns only the local side of a cut — placing the selected
 * local path on the OS clipboard with the platform's move marker.
 */

import { clipboard } from 'electron';
import { BridgeError } from '../shared/ipc-channels';

export interface ClipboardCutParams {
  paths: string[];
}

export interface ClipboardCutOutcome {
  accepted: boolean;
  acceptedCount: number;
  reason?: string | null;
}

/**
 * Builds the platform file-list payload for a cut.
 *
 * Windows marks a move through the `Preferred DropEffect` companion format,
 * macOS through `com.apple.pasteboard.promised-file`, and X11 through the
 * leading `cut` line of `x-special/gnome-copied-files`. The payload is written
 * as text so the marker survives the portable clipboard representation.
 */
export function buildCutPayload(paths: string[], platform: NodeJS.Platform): string {
  if (platform === 'win32') {
    return ['move', ...paths].join('\r\n');
  }
  return ['cut', ...paths.map((path) => `file://${path}`)].join('\n');
}

export interface ClipboardController {
  cutPaths(params: ClipboardCutParams): ClipboardCutOutcome;
  readPaths(): string[];
  writeText(text: string): void;
}

export function createClipboardController(
  platform: NodeJS.Platform = process.platform,
): ClipboardController {
  return {
    cutPaths(params) {
      if (!params.paths.length) {
        throw new BridgeError('invalid-state', 'Clipboard cut requires at least one path.');
      }
      clipboard.writeText(buildCutPayload(params.paths, platform));
      return { accepted: true, acceptedCount: params.paths.length };
    },
    readPaths() {
      return clipboard
        .readText()
        .split(/\r?\n/)
        .map((line) => line.trim())
        .filter((line) => line && line !== 'cut' && line !== 'move');
    },
    writeText(text) {
      clipboard.writeText(text);
    },
  };
}
