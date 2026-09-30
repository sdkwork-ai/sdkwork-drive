import React, { useCallback, useEffect, useRef, useState } from 'react';

/**
 * Accelerator recorder.
 *
 * Captures a key combination and normalizes it to the cross-platform spelling
 * (`CommandOrControl+Shift+X`) both the Tauri and Electron hosts normalize.
 *
 * Recording is deliberately keyboard-driven and non-modal-global: the field only
 * listens while focused, so the settings dialog never swallows application keys.
 */

export interface ShortcutRecorderProps {
  /** Current accelerator, or an empty string when unbound. */
  value: string;
  /** Called with the normalized accelerator; an empty string means "cleared". */
  onChange: (accelerator: string) => void;
  /** Optional label shown when nothing is bound. */
  placeholder?: string;
  /** Rendered for assistive technology and as the accessible name. */
  ariaLabel: string;
  disabled?: boolean;
  /** Disables the clear affordance (used for non-clearable built-ins). */
  clearable?: boolean;
}

/** Modifier key codes that never terminate a recording on their own. */
const MODIFIER_CODES = new Set([
  'ControlLeft',
  'ControlRight',
  'ShiftLeft',
  'ShiftRight',
  'AltLeft',
  'AltRight',
  'MetaLeft',
  'MetaRight',
]);

/** Maps a KeyboardEvent.code to the accelerator token both hosts accept. */
function codeToToken(code: string, key: string): string | null {
  if (code.startsWith('Key')) {
    return code.slice(3);
  }
  if (code.startsWith('Digit')) {
    return code.slice(5);
  }
  if (code.startsWith('Numpad')) {
    return `Numpad${code.slice(6)}`;
  }
  if (/^F\d{1,2}$/.test(code)) {
    return code;
  }

  const named: Record<string, string> = {
    Comma: ',',
    Period: '.',
    Slash: '/',
    Semicolon: ';',
    Quote: "'",
    BracketLeft: '[',
    BracketRight: ']',
    Backslash: '\\',
    Minus: '-',
    Equal: '=',
    Backquote: '`',
    Space: 'Space',
    Enter: 'Enter',
    NumpadEnter: 'Enter',
    Tab: 'Tab',
    Escape: 'Escape',
    Backspace: 'Backspace',
    Delete: 'Delete',
    Insert: 'Insert',
    Home: 'Home',
    End: 'End',
    PageUp: 'PageUp',
    PageDown: 'PageDown',
    ArrowUp: 'Up',
    ArrowDown: 'Down',
    ArrowLeft: 'Left',
    ArrowRight: 'Right',
    Escape2: 'Escape',
  };

  if (named[code]) {
    return named[code];
  }
  if (key.length === 1) {
    return key.toUpperCase();
  }
  return null;
}

/**
 * Builds the accelerator string from a keyboard event.
 *
 * `CommandOrControl` is emitted instead of a concrete Cmd/Ctrl token so one
 * stored binding works on every platform.
 */
export function acceleratorFromEvent(event: {
  code: string;
  key: string;
  ctrlKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
  metaKey: boolean;
}): string | null {
  const token = codeToToken(event.code, event.key);
  if (!token) {
    return null;
  }

  const parts: string[] = [];
  if (event.ctrlKey || event.metaKey) {
    parts.push('CommandOrControl');
  }
  if (event.altKey) {
    parts.push('Alt');
  }
  if (event.shiftKey) {
    parts.push('Shift');
  }
  parts.push(token);

  return parts.join('+');
}

/** Splits an accelerator into renderable key caps. */
export function acceleratorToKeyCaps(accelerator: string): string[] {
  if (!accelerator.trim()) {
    return [];
  }
  return accelerator.split('+').map((part) => part.trim()).filter(Boolean);
}

export function ShortcutRecorder({
  value,
  onChange,
  placeholder = '--',
  ariaLabel,
  disabled = false,
  clearable = true,
}: ShortcutRecorderProps) {
  const [recording, setRecording] = useState(false);
  const buttonRef = useRef<HTMLButtonElement | null>(null);

  const stopRecording = useCallback(() => setRecording(false), []);

  useEffect(() => {
    if (!recording) {
      return;
    }

    const handleKeyDown = (event: KeyboardEvent) => {
      // Modifier-only presses keep the recorder armed so the user can hold
      // Ctrl then Shift then X without the field committing halfway.
      if (MODIFIER_CODES.has(event.code)) {
        event.preventDefault();
        return;
      }

      event.preventDefault();
      event.stopPropagation();

      if (event.code === 'Escape') {
        stopRecording();
        buttonRef.current?.blur();
        return;
      }

      if (event.code === 'Backspace' || event.code === 'Delete') {
        onChange('');
        stopRecording();
        return;
      }

      const accelerator = acceleratorFromEvent(event);
      if (!accelerator) {
        return;
      }
      onChange(accelerator);
      stopRecording();
    };

    const handleBlur = () => stopRecording();

    window.addEventListener('keydown', handleKeyDown, true);
    window.addEventListener('blur', handleBlur);
    return () => {
      window.removeEventListener('keydown', handleKeyDown, true);
      window.removeEventListener('blur', handleBlur);
    };
  }, [onChange, recording, stopRecording]);

  const keyCaps = acceleratorToKeyCaps(value);

  return (
    <div className="flex items-center gap-2">
      <button
        ref={buttonRef}
        type="button"
        aria-label={ariaLabel}
        disabled={disabled}
        onClick={() => setRecording((current) => !current)}
        className={`flex h-9 min-w-[168px] items-center justify-center gap-1.5 rounded-xl border px-3 text-xs transition-all ${
          recording
            ? 'border-blue-500 bg-blue-500/10 text-blue-300'
            : 'border-white/10 bg-[#202020] text-gray-200 hover:border-white/20'
        } ${disabled ? 'cursor-not-allowed opacity-50' : 'cursor-pointer'}`}
      >
        {recording ? (
          <span className="text-[11px] font-medium">Press keys…</span>
        ) : keyCaps.length > 0 ? (
          keyCaps.map((cap, index) => (
            <React.Fragment key={`${cap}-${index}`}>
              {index > 0 && <span className="text-gray-600">+</span>}
              <kbd className="rounded-md border border-white/10 bg-black/30 px-1.5 py-0.5 font-mono text-[10px] text-gray-200">
                {cap}
              </kbd>
            </React.Fragment>
          ))
        ) : (
          <span className="text-[11px] text-gray-500">{placeholder}</span>
        )}
      </button>
      {clearable && value && !disabled && (
        <button
          type="button"
          onClick={() => onChange('')}
          className="rounded-lg px-2 py-1 text-[10px] text-gray-500 transition-colors hover:bg-white/5 hover:text-gray-300"
        >
          Clear
        </button>
      )}
    </div>
  );
}
