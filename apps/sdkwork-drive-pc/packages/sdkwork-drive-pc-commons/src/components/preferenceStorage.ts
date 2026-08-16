export interface PreferenceStorage {
  getItem(key: string): string | undefined;
  setItem(key: string, value: string): void;
}

export function readPreference(
  preferenceStorage: PreferenceStorage | undefined,
  key: string,
): string | undefined {
  try {
    return preferenceStorage?.getItem(key);
  } catch {
    return undefined;
  }
}

export function writePreference(
  preferenceStorage: PreferenceStorage | undefined,
  key: string,
  value: string,
): void {
  try {
    preferenceStorage?.setItem(key, value);
  } catch {
    // Preference persistence is best-effort and must not block UI state updates.
  }
}
