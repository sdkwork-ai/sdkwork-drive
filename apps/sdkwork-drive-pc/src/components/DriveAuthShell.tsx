import React, { useEffect, useState, type ReactNode } from 'react';
import { FileText, Moon, Sun, X } from 'lucide-react';

type AuthThemeMode = 'dark' | 'light';

function isDesktopRuntime(): boolean {
  return typeof window !== 'undefined' && !!(globalThis as Record<string, unknown>).__TAURI__;
}

// Desktop window control is delegated to the desktop package
// This component only handles the UI shell
export function DriveAuthShell({ children }: { children: ReactNode }) {
  const [themeMode, setThemeMode] = useState<AuthThemeMode>(() => {
    if (typeof window === 'undefined') return 'dark';
    return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
  });

  const isLightMode = themeMode === 'light';
  const shouldRenderDesktopHeader = isDesktopRuntime();

  useEffect(() => {
    document.documentElement.classList.toggle('light-mode', isLightMode);
    document.documentElement.style.colorScheme = themeMode;
  }, [themeMode, isLightMode]);

  const toggleTheme = () => {
    setThemeMode((current) => (current === 'light' ? 'dark' : 'light'));
  };

  const handleMinimize = () => {
    window.dispatchEvent(new CustomEvent('sdkwork-drive:window-control', { detail: { action: 'minimize' } }));
  };

  const handleToggleMaximize = () => {
    window.dispatchEvent(new CustomEvent('sdkwork-drive:window-control', { detail: { action: 'toggleMaximize' } }));
  };

  const handleClose = () => {
    window.dispatchEvent(new CustomEvent('sdkwork-drive:window-control', { detail: { action: 'close' } }));
  };

  return (
    <div className="sdkwork-drive-auth-shell">
      {shouldRenderDesktopHeader && (
        <header className="sdkwork-drive-auth-header drag-region">
          <div className="sdkwork-drive-auth-header-brand">
            <span className="sdkwork-drive-auth-header-mark">
              <FileText size={12} />
            </span>
            <span>SDKWork Drive</span>
          </div>
          <div className="sdkwork-drive-auth-header-center" />
          <div className="sdkwork-drive-auth-header-actions no-drag">
            <button
              aria-label={isLightMode ? 'Switch to dark mode' : 'Switch to light mode'}
              className="sdkwork-drive-auth-theme-button"
              onClick={toggleTheme}
              title={isLightMode ? 'Switch to dark mode' : 'Switch to light mode'}
              type="button"
            >
              {isLightMode ? <Moon size={14} /> : <Sun size={14} />}
            </button>
            <div className="sdkwork-drive-auth-window-controls">
              <button
                aria-label="Minimize window"
                className="sdkwork-drive-auth-window-button"
                onClick={handleMinimize}
                title="Minimize"
                type="button"
              >
                <svg aria-hidden="true" className="h-3.5 w-3.5" fill="none" viewBox="0 0 10 10">
                  <path d="M2 7H8" stroke="currentColor" strokeLinecap="square" strokeWidth="1" />
                </svg>
              </button>
              <button
                aria-label="Maximize window"
                className="sdkwork-drive-auth-window-button"
                onClick={handleToggleMaximize}
                title="Maximize"
                type="button"
              >
                <svg aria-hidden="true" className="h-3.5 w-3.5" fill="none" viewBox="0 0 10 10">
                  <path d="M2 2.5H8V8H2V2.5Z" stroke="currentColor" strokeWidth="1" />
                </svg>
              </button>
              <button
                aria-label="Close window"
                className="sdkwork-drive-auth-window-button sdkwork-drive-auth-window-button-danger"
                onClick={handleClose}
                title="Close"
                type="button"
              >
                <X size={14} />
              </button>
            </div>
          </div>
        </header>
      )}
      <main className="sdkwork-drive-auth-main">{children}</main>
    </div>
  );
}
