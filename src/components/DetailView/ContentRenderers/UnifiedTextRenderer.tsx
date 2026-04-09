import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { Check, Copy, Image as ImageIcon, Loader2 } from 'lucide-react';
import { Button } from '../../ui/button';
import { Badge } from '../../ui/badge';
import { ContentSubType } from '../../../types/clipboard';
import { useResolvedTheme } from '../../../hooks/useResolvedTheme';
import { defineMonacoThemes } from '../../../utils/monacoTheme';
import { generateCodeSnapshotDataUrl } from '../../../lib/codeSnapshot';
import { copyToClipboard } from '../../../stores/clipboardStore';
import MonacoEditor, { loader } from '@monaco-editor/react';
import * as monaco from 'monaco-editor';

// Configure Monaco to use local files instead of CDN
loader.config({ monaco });

interface UnifiedTextRendererProps {
  content: string;
  contentSubType: ContentSubType;
  metadata?: string | null;
  sessionKey?: string;
  onContentChange?: (value: string) => void;
}

// 内容类型到Monaco语言的映射
const getLanguageForContentType = (
  contentSubType: ContentSubType,
  metadata?: string | null
): string => {
  switch (contentSubType) {
    case 'code':
      // Get detected language from metadata
      if (metadata) {
        try {
          const parsed = JSON.parse(metadata);
          if (parsed.detected_language) {
            return parsed.detected_language;
          }
        } catch (e) {
          console.error('Failed to parse code metadata:', e);
        }
      }
      return 'plaintext';
    case 'json':
      return 'json';
    case 'markdown':
      return 'markdown';
    case 'command':
      return 'shell';
    case 'plain_text':
    default:
      return 'plaintext';
  }
};

// 内容类型到显示名称的映射
const getDisplayNameForContentType = (
  contentSubType: ContentSubType,
  t: (key: string) => string
): string => {
  switch (contentSubType) {
    case 'code':
      return t('codeEditor.code');
    case 'json':
      return t('codeEditor.json');
    case 'markdown':
      return t('codeEditor.markdown');
    case 'command':
      return t('codeEditor.command');
    case 'plain_text':
    default:
      return t('codeEditor.text');
  }
};

export function UnifiedTextRenderer({
  content,
  contentSubType,
  metadata,
  sessionKey,
  onContentChange,
}: UnifiedTextRendererProps) {
  const { t } = useTranslation(['common']);
  const [editedContent, setEditedContent] = useState(content);
  const [isCopied, setIsCopied] = useState(false);
  const [isGeneratingSnapshot, setIsGeneratingSnapshot] = useState(false);
  const [isSnapshotCopied, setIsSnapshotCopied] = useState(false);
  const resolvedTheme = useResolvedTheme();

  // 切换条目会带来新的 sessionKey；即使原始内容相同，也必须重置本地 workbench。
  useEffect(() => {
    setEditedContent(content);
    setIsSnapshotCopied(false);
    onContentChange?.(content);
  }, [content, onContentChange, sessionKey]);

  const language = getLanguageForContentType(contentSubType, metadata);
  const displayName = getDisplayNameForContentType(contentSubType, t);
  const monacoTheme = resolvedTheme === 'dark' ? 'clipboard-dark' : 'clipboard-light';
  const editorHeight =
    contentSubType === 'plain_text' ? 'clamp(280px, 38vh, 640px)' : 'clamp(360px, 52vh, 920px)';
  const showLineNumbers = contentSubType !== 'plain_text';
  const copyButtonLabel = isCopied ? t('codeEditor.copied') : t('codeEditor.copy');
  const showSnapshotButton = contentSubType === 'code' || contentSubType === 'command';
  const snapshotButtonLabel = isGeneratingSnapshot
    ? t('codeEditor.snapshotting')
    : isSnapshotCopied
      ? t('codeEditor.snapshotCopied')
      : t('codeEditor.snapshot');

  const handleCopy = async () => {
    try {
      await copyToClipboard(editedContent);
      setIsCopied(true);
      setTimeout(() => setIsCopied(false), 2000);
    } catch (error) {
      console.error(t('codeEditor.copyFailed'), error);
    }
  };

  const handleEditorChange = (value?: string) => {
    const nextValue = value || '';
    setEditedContent(nextValue);
    onContentChange?.(nextValue);
  };

  const handleCopySnapshot = async () => {
    setIsGeneratingSnapshot(true);

    try {
      const snapshotDataUrl = await generateCodeSnapshotDataUrl({
        content: editedContent,
        language,
        theme: resolvedTheme === 'dark' ? 'dark' : 'light',
        title: displayName,
        showLineNumbers,
      });

      await invoke('copy_converted_image', {
        base64Data: snapshotDataUrl,
        skipRecording: true,
      });

      setIsSnapshotCopied(true);
      window.setTimeout(() => setIsSnapshotCopied(false), 2000);
    } catch (error) {
      console.error(t('codeEditor.snapshotFailed', { error: String(error) }), error);
      alert(t('codeEditor.snapshotFailed', { error: String(error) }));
    } finally {
      setIsGeneratingSnapshot(false);
    }
  };

  return (
    <div
      id="text-renderer"
      className="flex min-h-[320px] min-w-0 flex-1 flex-col overflow-hidden min-[1200px]:min-h-[420px]"
    >
      <div
        id="text-renderer-header"
        className="flex flex-wrap items-center justify-between gap-3 border-b border-border/70 px-3 py-2.5 min-[1200px]:px-4 min-[1200px]:py-3"
      >
        <div id="text-renderer-badges" className="flex min-w-0 flex-wrap items-center gap-2">
          <Badge variant="secondary">{displayName}</Badge>
          {language !== 'plaintext' && (
            <Badge variant="outline" className="text-xs">
              {language}
            </Badge>
          )}
        </div>
        <div className="flex items-center gap-2">
          {showSnapshotButton && (
            <Button
              id="text-renderer-snapshot-btn"
              onClick={handleCopySnapshot}
              size="sm"
              variant="outline"
              disabled={isGeneratingSnapshot || editedContent.length === 0}
              aria-label={snapshotButtonLabel}
              title={snapshotButtonLabel}
              className="h-8 rounded-lg px-2.5 text-xs"
            >
              {isGeneratingSnapshot ? (
                <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
              ) : isSnapshotCopied ? (
                <Check className="mr-1.5 h-3.5 w-3.5" />
              ) : (
                <ImageIcon className="mr-1.5 h-3.5 w-3.5" />
              )}
              <span>{snapshotButtonLabel}</span>
            </Button>
          )}
          <Button
            id="text-renderer-copy-btn"
            onClick={handleCopy}
            size="icon"
            variant="outline"
            aria-label={copyButtonLabel}
            title={copyButtonLabel}
            className="h-8 w-8 rounded-lg"
          >
            {isCopied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
          </Button>
        </div>
      </div>

      <div id="text-renderer-content" className="flex min-h-0 flex-1 flex-col overflow-hidden">
        <div
          id="text-renderer-editor-container"
          className="flex min-h-0 flex-1 border-t"
          style={{ height: editorHeight }}
        >
          <MonacoEditor
            key={`${sessionKey ?? 'default'}:${language}:${resolvedTheme}`}
            height="100%"
            language={language}
            value={editedContent}
            onChange={handleEditorChange}
            theme={monacoTheme}
            beforeMount={(monaco) => {
              defineMonacoThemes(monaco);
            }}
            options={{
              readOnly: false,
              minimap: { enabled: false },
              scrollBeyondLastLine: false,
              wordWrap: 'on',
              fontSize: 13,
              lineNumbers: showLineNumbers ? 'on' : 'off',
              lineNumbersMinChars: showLineNumbers ? 3 : 0,
              glyphMargin: false,
              lineDecorationsWidth: showLineNumbers ? 10 : 0,
              renderWhitespace: 'selection',
              automaticLayout: true,
              padding: { top: 18, bottom: 20 },
              tabSize: 2,
              insertSpaces: true,
              quickSuggestions: true,
              suggestOnTriggerCharacters: true,
              acceptSuggestionOnEnter: 'on',
              wordBasedSuggestions: 'currentDocument',
              parameterHints: { enabled: true },
              folding: language !== 'plaintext',
              foldingHighlight: language !== 'plaintext',
              unfoldOnClickAfterEndOfLine: true,
              selectOnLineNumbers: showLineNumbers,
              contextmenu: true,
              cursorBlinking: 'blink',
              cursorSmoothCaretAnimation: 'on',
            }}
          />
        </div>
      </div>
    </div>
  );
}
