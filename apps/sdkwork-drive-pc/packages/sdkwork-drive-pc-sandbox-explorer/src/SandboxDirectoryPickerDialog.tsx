import { Copy, Maximize2, Minimize2, Square, X } from 'lucide-react';
import { useEffect, useId, useRef, useState } from 'react';
import type { SandboxExplorerPort, SandboxSelection } from './contracts';
import { SandboxExplorerView } from './SandboxExplorerView';

type DesktopPlatform = 'windows' | 'macos' | 'linux';

function resolveDesktopPlatform(): DesktopPlatform {
  if (typeof navigator === 'undefined') return 'windows';
  const platform = `${navigator.userAgent} ${navigator.platform}`.toLocaleLowerCase();
  if (platform.includes('mac')) return 'macos';
  if (platform.includes('linux')) return 'linux';
  return 'windows';
}

export interface SandboxDirectoryPickerDialogProps {
  readonly open: boolean;
  readonly port?: SandboxExplorerPort;
  readonly title?: string;
  readonly onCancel: () => void;
  readonly onDirectorySelected: (selection: SandboxSelection) => void;
}

export function SandboxDirectoryPickerDialog({
  open,
  port,
  title = 'Select server directory',
  onCancel,
  onDirectorySelected,
}: SandboxDirectoryPickerDialogProps) {
  const titleId = useId();
  const closeButtonRef = useRef<HTMLButtonElement>(null);
  const dialogRef = useRef<HTMLElement>(null);
  const onCancelRef = useRef(onCancel);
  const [maximized, setMaximized] = useState(true);
  const platform = resolveDesktopPlatform();

  useEffect(() => {
    onCancelRef.current = onCancel;
  }, [onCancel]);

  useEffect(() => {
    if (!open) return undefined;
    setMaximized(true);
    const previouslyFocused = document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null;
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onCancelRef.current();
    };
    globalThis.addEventListener('keydown', handleKeyDown);
    closeButtonRef.current?.focus();
    return () => {
      globalThis.removeEventListener('keydown', handleKeyDown);
      previouslyFocused?.focus();
    };
  }, [open]);

  if (!open) return null;

  return (
    <div
      className={`sdkwork-sandbox-dialog-backdrop${maximized ? ' is-maximized' : ''}`}
      role="presentation"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) onCancel();
      }}
    >
      <section
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className={`sdkwork-sandbox-dialog sdkwork-sandbox-dialog--${platform}${maximized ? ' is-maximized' : ''}`}
        onKeyDown={(event) => {
          if (event.key !== 'Tab') return;
          const focusable = dialogRef.current?.querySelectorAll<HTMLElement>(
            'button:not([disabled]), select:not([disabled]), input:not([disabled]), [href], [tabindex]:not([tabindex="-1"])',
          );
          if (!focusable?.length) {
            event.preventDefault();
            return;
          }
          const first = focusable[0];
          const last = focusable[focusable.length - 1];
          if (event.shiftKey && document.activeElement === first) {
            event.preventDefault();
            last?.focus();
          } else if (!event.shiftKey && document.activeElement === last) {
            event.preventDefault();
            first?.focus();
          }
        }}
      >
        <header
          className="sdkwork-sandbox-dialog__header"
          onDoubleClick={(event) => {
            if ((event.target as HTMLElement).closest('button')) return;
            setMaximized((current) => !current);
          }}
        >
          <div className="sdkwork-sandbox-dialog__window-controls">
            {platform !== 'macos' && (
              <button
                type="button"
                title={maximized ? 'Restore' : 'Maximize'}
                aria-label={maximized ? 'Restore' : 'Maximize'}
                aria-pressed={maximized}
                className="sdkwork-sandbox-dialog__window-button sdkwork-sandbox-dialog__maximize"
                onClick={() => setMaximized((current) => !current)}
              >
                {maximized ? <Copy size={13} /> : <Square size={12} />}
              </button>
            )}
            <button
              ref={closeButtonRef}
              type="button"
              title="Close"
              aria-label="Close"
              className="sdkwork-sandbox-dialog__window-button sdkwork-sandbox-dialog__close"
              onClick={onCancel}
            >
              <X size={17} />
            </button>
            {platform === 'macos' && (
              <button
                type="button"
                title={maximized ? 'Restore' : 'Maximize'}
                aria-label={maximized ? 'Restore' : 'Maximize'}
                aria-pressed={maximized}
                className="sdkwork-sandbox-dialog__window-button sdkwork-sandbox-dialog__maximize"
                onClick={() => setMaximized((current) => !current)}
              >
                {maximized ? <Minimize2 size={15} /> : <Maximize2 size={15} />}
              </button>
            )}
          </div>
          <h2 id={titleId}>{title}</h2>
        </header>
        <SandboxExplorerView
          mode="select-directory"
          port={port}
          onDirectorySelected={onDirectorySelected}
        />
      </section>
    </div>
  );
}
