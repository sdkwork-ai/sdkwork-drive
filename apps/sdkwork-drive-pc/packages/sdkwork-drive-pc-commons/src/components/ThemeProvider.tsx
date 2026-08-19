import React, { createContext, useContext, useEffect, useLayoutEffect, useState } from 'react';
import {
  readPreference,
  writePreference,
  type PreferenceStorage,
} from './preferenceStorage';

export type Theme = 'dark' | 'light' | 'system';

export type ColorScheme = 'dark' | 'light';

interface ThemeProviderProps {
  children: React.ReactNode;
  defaultTheme?: Theme;
  preferenceStorage?: PreferenceStorage;
  storageKey?: string;
  resolveHostColorScheme?: () => ColorScheme;
  subscribeHostColorScheme?: (listener: (scheme: ColorScheme) => void) => () => void;
}

interface ThemeProviderState {
  theme: Theme;
  setTheme: (theme: Theme) => void;
}

const initialState: ThemeProviderState = {
  theme: 'system',
  setTheme: () => null,
};

const ThemeProviderContext = createContext<ThemeProviderState>(initialState);

function resolveSystemColorScheme(): ColorScheme {
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function resolveStandaloneTheme(
  defaultTheme: Theme,
  preferenceStorage?: PreferenceStorage,
  storageKey?: string,
): Theme {
  return normalizeTheme(readPreference(preferenceStorage, storageKey)) ?? defaultTheme;
}

function resolveColorScheme(theme: Theme): ColorScheme {
  if (theme === 'system') {
    return resolveSystemColorScheme();
  }
  return theme;
}

export function ThemeProvider({
  children,
  defaultTheme = 'system',
  preferenceStorage,
  storageKey = 'sdkwork-ui-theme',
  resolveHostColorScheme,
  subscribeHostColorScheme,
  ...props
}: ThemeProviderProps) {
  const hostManaged = resolveHostColorScheme !== undefined;
  const [theme, setThemeState] = useState<Theme>(() => {
    if (hostManaged) {
      return resolveHostColorScheme();
    }
    return resolveStandaloneTheme(defaultTheme, preferenceStorage, storageKey);
  });

  useEffect(() => {
    if (!subscribeHostColorScheme) {
      return undefined;
    }

    return subscribeHostColorScheme((nextScheme) => {
      setThemeState(nextScheme);
    });
  }, [subscribeHostColorScheme]);

  useEffect(() => {
    if (hostManaged) {
      return undefined;
    }

    const root = window.document.documentElement;
    root.classList.remove('light', 'dark');
    root.classList.add(resolveColorScheme(theme));
    return undefined;
  }, [hostManaged, theme]);

  const scheme = resolveColorScheme(theme);

  useLayoutEffect(() => {
    if (!hostManaged) {
      return undefined;
    }
    const root = document.documentElement;
    const previousDark = root.classList.contains('dark');
    const previousLight = root.classList.contains('light');
    const previousLightMode = root.classList.contains('light-mode');
    const previousSdkColorMode = root.getAttribute('data-sdk-color-mode');
    root.classList.toggle('dark', scheme === 'dark');
    root.classList.toggle('light', scheme === 'light');
    root.classList.toggle('light-mode', scheme === 'light');
    root.setAttribute('data-sdk-color-mode', scheme);
    return () => {
      root.classList.toggle('dark', previousDark);
      root.classList.toggle('light', previousLight);
      root.classList.toggle('light-mode', previousLightMode);
      if (previousSdkColorMode === null) root.removeAttribute('data-sdk-color-mode');
      else root.setAttribute('data-sdk-color-mode', previousSdkColorMode);
    };
  }, [hostManaged, scheme]);

  const value = {
    theme,
    setTheme: (nextTheme: Theme) => {
      if (hostManaged) {
        return;
      }
      writePreference(preferenceStorage, storageKey, nextTheme);
      setThemeState(nextTheme);
    },
  };

  const content = (
    <ThemeProviderContext.Provider {...props} value={value}>
      {children}
    </ThemeProviderContext.Provider>
  );

  if (!hostManaged) {
    return content;
  }

  return (
    <div className={`flex h-full min-h-0 w-full min-w-0 flex-col${scheme === 'dark' ? ' dark' : ''}`} data-sdk-color-mode={scheme}>
      {content}
    </div>
  );
}

export const useTheme = () => {
  const context = useContext(ThemeProviderContext);

  if (context === undefined)
    throw new Error('useTheme must be used within a ThemeProvider');

  return context;
};

function normalizeTheme(value: string | undefined): Theme | undefined {
  if (value === 'dark' || value === 'light' || value === 'system') {
    return value;
  }
  return undefined;
}
