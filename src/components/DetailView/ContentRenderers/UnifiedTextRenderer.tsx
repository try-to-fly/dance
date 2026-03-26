import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Copy } from 'lucide-react';
import { Button } from '../../ui/button';
import { Badge } from '../../ui/badge';
import { ContentSubType } from '../../../types/clipboard';
import { useResolvedTheme } from '../../../hooks/useResolvedTheme';
import { defineMonacoThemes } from '../../../utils/monacoTheme';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import MonacoEditor, { loader } from '@monaco-editor/react';
import * as monaco from 'monaco-editor';

// Configure Monaco to use local files instead of CDN
loader.config({ monaco });

interface UnifiedTextRendererProps {
  content: string;
  contentSubType: ContentSubType;
  metadata?: string | null;
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
}: UnifiedTextRendererProps) {
  const { t } = useTranslation(['common']);
  const [editedContent, setEditedContent] = useState(content);
  const [isCopied, setIsCopied] = useState(false);
  const resolvedTheme = useResolvedTheme();

  // 当content props变化时，更新编辑器内容
  useEffect(() => {
    setEditedContent(content);
  }, [content]);

  const language = getLanguageForContentType(contentSubType, metadata);
  const displayName = getDisplayNameForContentType(contentSubType, t);
  const monacoTheme = resolvedTheme === 'dark' ? 'clipboard-dark' : 'clipboard-light';
  const editorHeight =
    contentSubType === 'plain_text' ? 'clamp(280px, 38vh, 640px)' : 'clamp(360px, 52vh, 920px)';
  const showLineNumbers = contentSubType !== 'plain_text';

  const handleCopy = async () => {
    try {
      await writeText(editedContent);
      setIsCopied(true);
      setTimeout(() => setIsCopied(false), 2000);
    } catch (error) {
      console.error(t('codeEditor.copyFailed'), error);
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
          <Button
            id="text-renderer-copy-btn"
            onClick={handleCopy}
            size="sm"
            variant="outline"
            className="h-8 rounded-lg"
          >
            <Copy className="mr-2 h-4 w-4" />
            {isCopied ? t('codeEditor.copied') : t('codeEditor.copy')}
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
            key={`${language}-${resolvedTheme}-${content.substring(0, 50)}`}
            height="100%"
            language={language}
            value={editedContent}
            onChange={(value) => setEditedContent(value || '')}
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
