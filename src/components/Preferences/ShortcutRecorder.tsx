import { useState, useEffect, useRef } from 'react';
import { Keyboard, RotateCcw, Check, X } from 'lucide-react';
import { Button } from '../ui/button';
import { cn } from '../../lib/utils';

interface ShortcutRecorderProps {
  value: string;
  onChange: (shortcut: string) => void;
  onValidate?: (shortcut: string) => Promise<boolean>;
}

interface RecordedKeys {
  key: string;
  modifiers: {
    cmd: boolean;
    ctrl: boolean;
    alt: boolean;
    shift: boolean;
  };
}

const MAC_KEY_SYMBOLS: Record<string, string> = {
  cmdorctrl: '⌘',
  commandorcontrol: '⌘',
  cmd: '⌘',
  ctrl: '⌃',
  alt: '⌥',
  shift: '⇧',
  meta: '⌘',
  control: '⌃',
  option: '⌥',
  enter: '↵',
  return: '↵',
  escape: '⎋',
  esc: '⎋',
  tab: '⇥',
  space: '␣',
  backspace: '⌫',
  delete: '⌦',
  arrowup: '↑',
  arrowdown: '↓',
  arrowleft: '←',
  arrowright: '→',
  up: '↑',
  down: '↓',
  left: '←',
  right: '→',
};

const MODIFIER_KEYS = new Set(['meta', 'control', 'alt', 'shift', 'cmd', 'ctrl']);

const CODE_TO_SHORTCUT_KEY: Record<string, string> = {
  Space: 'Space',
  Tab: 'Tab',
  Enter: 'Enter',
  NumpadEnter: 'Enter',
  Escape: 'Escape',
  Backspace: 'Backspace',
  Delete: 'Delete',
  ArrowUp: 'Up',
  ArrowDown: 'Down',
  ArrowLeft: 'Left',
  ArrowRight: 'Right',
  Home: 'Home',
  End: 'End',
  PageUp: 'PageUp',
  PageDown: 'PageDown',
};

const getShortcutKeyFromCode = (code: string): string | null => {
  if (code.startsWith('Key') && code.length === 4) {
    return code.slice(3);
  }

  if (code.startsWith('Digit') && code.length === 6) {
    return code.slice(5);
  }

  if (code.startsWith('Numpad') && code.length === 7) {
    const digit = code.slice(6);
    if (/^[0-9]$/.test(digit)) {
      return digit;
    }
  }

  return CODE_TO_SHORTCUT_KEY[code] ?? null;
};

export function ShortcutRecorder({ value, onChange, onValidate }: ShortcutRecorderProps) {
  const [isRecording, setIsRecording] = useState(false);
  const [recordedKeys, setRecordedKeys] = useState<RecordedKeys | null>(null);
  const [isValidating, setIsValidating] = useState(false);
  const [validationError, setValidationError] = useState<string | null>(null);
  const inputRef = useRef<HTMLDivElement>(null);

  // Convert shortcut string to display format with symbols
  const formatShortcut = (shortcut: string): string => {
    if (!shortcut) return '';

    return shortcut
      .split('+')
      .map((part) => {
        const key = part.toLowerCase().trim();
        return MAC_KEY_SYMBOLS[key] || part.toUpperCase();
      })
      .join(' + ');
  };

  // Convert recorded keys to shortcut string
  const keysToShortcut = (keys: RecordedKeys): string => {
    const parts: string[] = [];

    if (keys.modifiers.cmd || keys.modifiers.ctrl) {
      parts.push('CmdOrCtrl');
    }
    if (keys.modifiers.alt) {
      parts.push('Alt');
    }
    if (keys.modifiers.shift) {
      parts.push('Shift');
    }

    parts.push(keys.key);

    return parts.join('+');
  };

  // Handle key recording
  const handleKeyDown = (e: KeyboardEvent) => {
    if (!isRecording) return;

    e.preventDefault();
    e.stopPropagation();

    const key = e.key.toLowerCase();

    // Skip modifier-only keys
    if (MODIFIER_KEYS.has(key)) {
      return;
    }

    const normalizedKey = getShortcutKeyFromCode(e.code);
    if (!normalizedKey) {
      setRecordedKeys(null);
      setValidationError('该按键暂不支持作为快捷键主键，请换一个键');
      return;
    }

    setValidationError(null);

    const recorded: RecordedKeys = {
      key: normalizedKey,
      modifiers: {
        cmd: e.metaKey,
        ctrl: e.ctrlKey,
        alt: e.altKey,
        shift: e.shiftKey,
      },
    };

    setRecordedKeys(recorded);
  };

  useEffect(() => {
    if (isRecording) {
      document.addEventListener('keydown', handleKeyDown, true);
      return () => {
        document.removeEventListener('keydown', handleKeyDown, true);
      };
    }
  }, [isRecording]);

  const startRecording = () => {
    setIsRecording(true);
    setRecordedKeys(null);
    setValidationError(null);
    inputRef.current?.focus();
  };

  const cancelRecording = () => {
    setIsRecording(false);
    setRecordedKeys(null);
    setValidationError(null);
  };

  const confirmShortcut = async () => {
    if (!recordedKeys) return;

    const newShortcut = keysToShortcut(recordedKeys);

    // Validate the shortcut
    if (onValidate) {
      setIsValidating(true);
      try {
        const isValid = await onValidate(newShortcut);
        if (!isValid) {
          setValidationError('This shortcut conflicts with system shortcuts or is invalid');
          setIsValidating(false);
          return;
        }
      } catch (error) {
        console.error('Failed to validate shortcut:', error);
        setValidationError('Failed to validate shortcut');
        setIsValidating(false);
        return;
      }
      setIsValidating(false);
    }

    onChange(newShortcut);
    setIsRecording(false);
    setRecordedKeys(null);
    setValidationError(null);
  };

  const currentDisplay = recordedKeys ? formatShortcut(keysToShortcut(recordedKeys)) : '';
  const hasValidKeys =
    recordedKeys &&
    (recordedKeys.modifiers.cmd || recordedKeys.modifiers.ctrl || recordedKeys.modifiers.alt) &&
    recordedKeys.key !== '';

  return (
    <div className="space-y-3">
      {!isRecording ? (
        <div className="flex items-center justify-between p-3 bg-secondary rounded-lg border">
          <div className="flex items-center gap-2 font-mono text-sm">
            <Keyboard className="w-4 h-4" />
            <span>{formatShortcut(value) || '未设置'}</span>
          </div>
          <Button variant="outline" size="sm" onClick={startRecording}>
            更改
          </Button>
        </div>
      ) : (
        <div className="space-y-3">
          <div
            ref={inputRef}
            className={cn(
              'flex items-center justify-center p-4 border-2 rounded-lg min-h-[60px] font-mono text-sm transition-colors',
              'border-primary bg-primary/10 text-primary',
              'focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2'
            )}
            tabIndex={0}
          >
            {currentDisplay || '请按下快捷键组合...'}
          </div>

          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm" onClick={cancelRecording} title="取消">
              <X className="w-4 h-4" />
            </Button>

            <Button
              variant="outline"
              size="sm"
              onClick={() => setRecordedKeys(null)}
              title="重新录制"
              disabled={!recordedKeys}
            >
              <RotateCcw className="w-4 h-4" />
            </Button>

            <Button
              variant="default"
              size="sm"
              onClick={confirmShortcut}
              title="确认"
              disabled={!hasValidKeys || isValidating}
            >
              {isValidating ? (
                <div className="animate-spin rounded-full h-4 w-4 border-2 border-background border-t-transparent" />
              ) : (
                <Check className="w-4 h-4" />
              )}
            </Button>
          </div>
        </div>
      )}

      {validationError && <p className="text-sm text-destructive">{validationError}</p>}

      {isRecording && (
        <p className="text-sm text-muted-foreground italic">
          快捷键必须包含至少一个修饰键（⌘、⌃、⌥）
        </p>
      )}
    </div>
  );
}
