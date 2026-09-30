import React, { createContext, useCallback, useContext, useEffect, useState } from 'react';
import {
  readPreference,
  writePreference,
  type PreferenceStorage,
} from './preferenceStorage';

/** Command ids shared by the tray menu and the shortcut binding table. */
export type DrivePcShortcutCommandId =
  | 'drive.openSettings'
  | 'drive.showWindow'
  | 'drive.cutSelection'
  | 'drive.pasteSelection'
  | 'drive.refresh'
  | 'drive.toggleTray';

/** Accelerator per command id; empty string means "unbound". */
export type DrivePcShortcutBindings = Record<DrivePcShortcutCommandId, string>;

export interface DrivePcPreferences {
  compactMode: boolean;
  transferStartAlert: boolean;
  systemDialogVerification: boolean;
  malwareCheckBanners: boolean;
  deleteShareConfirm: boolean;
  previewCacheAutoClear: boolean;
  /** Master switch for OS-level global shortcuts. */
  globalShortcutsEnabled: boolean;
  /** Show the tray icon while the app is running. */
  trayEnabled: boolean;
  /** Minimize to tray instead of quitting on window close. */
  minimizeToTrayOnClose: boolean;
  /** Per-command accelerator bindings. */
  shortcutBindings: DrivePcShortcutBindings;
}

const STORAGE_KEY = 'sdkwork.drive.pc.preferences.v1';

/**
 * Default accelerators.
 *
 * `CommandOrControl` maps to Cmd on macOS and Ctrl elsewhere, which is the
 * spelling both the Tauri and Electron hosts normalize.
 */
export const DEFAULT_SHORTCUT_BINDINGS: DrivePcShortcutBindings = {
  'drive.openSettings': 'CommandOrControl+,',
  'drive.showWindow': 'CommandOrControl+Shift+W',
  'drive.cutSelection': 'CommandOrControl+Shift+X',
  'drive.pasteSelection': 'CommandOrControl+Shift+V',
  'drive.refresh': 'CommandOrControl+Shift+R',
  'drive.toggleTray': 'CommandOrControl+Shift+T',
};

const DEFAULT_PREFERENCES: DrivePcPreferences = {
  compactMode: false,
  transferStartAlert: true,
  systemDialogVerification: false,
  malwareCheckBanners: true,
  deleteShareConfirm: true,
  previewCacheAutoClear: true,
  globalShortcutsEnabled: true,
  trayEnabled: true,
  minimizeToTrayOnClose: true,
  shortcutBindings: { ...DEFAULT_SHORTCUT_BINDINGS },
};

/** Preference keys that must not be persisted into foreign SDK identifiers. */
function normalizeBindings(raw: unknown): DrivePcShortcutBindings {
  const merged: DrivePcShortcutBindings = { ...DEFAULT_SHORTCUT_BINDINGS };
  if (raw && typeof raw === 'object') {
    for (const [key, value] of Object.entries(raw as Record<string, unknown>)) {
      if (key in merged && typeof value === 'string') {
        merged[key as DrivePcShortcutCommandId] = value;
      }
    }
  }
  return merged;
}

function parsePreferences(raw: string | undefined): DrivePcPreferences {
  if (!raw) {
    return { ...DEFAULT_PREFERENCES, shortcutBindings: { ...DEFAULT_SHORTCUT_BINDINGS } };
  }
  try {
    const parsed = JSON.parse(raw) as Partial<DrivePcPreferences>;
    return {
      ...DEFAULT_PREFERENCES,
      ...parsed,
      shortcutBindings: normalizeBindings(parsed.shortcutBindings),
    };
  } catch {
    return { ...DEFAULT_PREFERENCES, shortcutBindings: { ...DEFAULT_SHORTCUT_BINDINGS } };
  }
}

export function readDrivePcPreferences(
  preferenceStorage?: PreferenceStorage,
): DrivePcPreferences {
  return parsePreferences(readPreference(preferenceStorage, STORAGE_KEY));
}

export function writeDrivePcPreferences(
  patch: Partial<DrivePcPreferences>,
  preferenceStorage?: PreferenceStorage,
): DrivePcPreferences {
  const next = {
    ...readDrivePcPreferences(preferenceStorage),
    ...patch,
  };
  writePreference(preferenceStorage, STORAGE_KEY, JSON.stringify(next));
  return next;
}

/**
 * Rewrites a single accelerator binding.
 *
 * Passing an empty accelerator unbinds the command rather than throwing, so the
 * Settings Center can offer an explicit "clear" affordance.
 */
export function applyShortcutBinding(
  bindings: DrivePcShortcutBindings,
  commandId: DrivePcShortcutCommandId,
  accelerator: string,
): DrivePcShortcutBindings {
  return {
    ...bindings,
    [commandId]: accelerator.trim(),
  };
}

interface DrivePcPreferencesContextValue {
  preferences: DrivePcPreferences;
  updatePreferences: (patch: Partial<DrivePcPreferences>) => DrivePcPreferences;
  updateShortcutBinding: (
    commandId: DrivePcShortcutCommandId,
    accelerator: string,
  ) => DrivePcPreferences;
  resetShortcutBindings: () => DrivePcPreferences;
}

const DrivePcPreferencesContext = createContext<DrivePcPreferencesContextValue | undefined>(
  undefined,
);

export function DrivePcPreferencesProvider({
  children,
  preferenceStorage,
}: {
  children: React.ReactNode;
  preferenceStorage?: PreferenceStorage;
}) {
  const [preferences, setPreferences] = useState<DrivePcPreferences>(() =>
    readDrivePcPreferences(preferenceStorage),
  );

  useEffect(() => {
    setPreferences(readDrivePcPreferences(preferenceStorage));
  }, [preferenceStorage]);

  const updatePreferences = useCallback(
    (patch: Partial<DrivePcPreferences>) => {
      const next = writeDrivePcPreferences(patch, preferenceStorage);
      setPreferences(next);
      return next;
    },
    [preferenceStorage],
  );

  const updateShortcutBinding = useCallback(
    (commandId: DrivePcShortcutCommandId, accelerator: string) => {
      const current = readDrivePcPreferences(preferenceStorage);
      const next = writeDrivePcPreferences(
        {
          shortcutBindings: applyShortcutBinding(
            current.shortcutBindings,
            commandId,
            accelerator,
          ),
        },
        preferenceStorage,
      );
      setPreferences(next);
      return next;
    },
    [preferenceStorage],
  );

  const resetShortcutBindings = useCallback(() => {
    const next = writeDrivePcPreferences(
      { shortcutBindings: { ...DEFAULT_SHORTCUT_BINDINGS } },
      preferenceStorage,
    );
    setPreferences(next);
    return next;
  }, [preferenceStorage]);

  return (
    <DrivePcPreferencesContext.Provider
      value={{ preferences, updatePreferences, updateShortcutBinding, resetShortcutBindings }}
    >
      {children}
    </DrivePcPreferencesContext.Provider>
  );
}

export function useDrivePcPreferences(): DrivePcPreferencesContextValue {
  const context = useContext(DrivePcPreferencesContext);
  if (!context) {
    throw new Error('useDrivePcPreferences must be used within a DrivePcPreferencesProvider');
  }
  return context;
}
