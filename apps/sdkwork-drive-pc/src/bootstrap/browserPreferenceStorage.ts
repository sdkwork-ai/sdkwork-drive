import type { PreferenceStorage } from 'sdkwork-drive-pc-commons';

export function createBrowserPreferenceStorage(): PreferenceStorage | undefined {
  if (typeof window === 'undefined') {
    return undefined;
  }

  return {
    getItem(key) {
      return window.localStorage.getItem(key) ?? undefined;
    },
    setItem(key, value) {
      window.localStorage.setItem(key, value);
    },
  };
}
