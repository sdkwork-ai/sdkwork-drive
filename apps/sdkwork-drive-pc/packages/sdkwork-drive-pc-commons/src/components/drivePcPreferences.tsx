import React, { createContext, useCallback, useContext, useEffect, useState } from 'react';
import {
  readPreference,
  writePreference,
  type PreferenceStorage,
} from './preferenceStorage';

export interface DrivePcPreferences {
  compactMode: boolean;
  transferStartAlert: boolean;
  systemDialogVerification: boolean;
  malwareCheckBanners: boolean;
  deleteShareConfirm: boolean;
  previewCacheAutoClear: boolean;
}

const STORAGE_KEY = 'sdkwork.drive.pc.preferences.v1';

const DEFAULT_PREFERENCES: DrivePcPreferences = {
  compactMode: false,
  transferStartAlert: true,
  systemDialogVerification: false,
  malwareCheckBanners: true,
  deleteShareConfirm: true,
  previewCacheAutoClear: true,
};

function parsePreferences(raw: string | undefined): DrivePcPreferences {
  if (!raw) {
    return { ...DEFAULT_PREFERENCES };
  }
  try {
    const parsed = JSON.parse(raw) as Partial<DrivePcPreferences>;
    return {
      ...DEFAULT_PREFERENCES,
      ...parsed,
    };
  } catch {
    return { ...DEFAULT_PREFERENCES };
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

interface DrivePcPreferencesContextValue {
  preferences: DrivePcPreferences;
  updatePreferences: (patch: Partial<DrivePcPreferences>) => DrivePcPreferences;
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

  return (
    <DrivePcPreferencesContext.Provider value={{ preferences, updatePreferences }}>
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
