import { useState, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { Copy, Braces, List } from 'lucide-react';
import { JsonView, darkStyles, defaultStyles } from 'react-json-view-lite';
import 'react-json-view-lite/dist/index.css';
import { Button } from '../../ui/button';
import { Badge } from '../../ui/badge';
import { useResolvedTheme } from '../../../hooks/useResolvedTheme';
import { defineMonacoThemes } from '../../../utils/monacoTheme';
import { copyToClipboard } from '../../../stores/clipboardStore';
import MonacoEditor, { loader } from '@monaco-editor/react';
import * as monaco from 'monaco-editor';

loader.config({ monaco });

interface JsonRendererProps {
  content: string;
}

type ViewMode = 'tree' | 'code';

export function JsonRenderer({ content }: JsonRendererProps) {
  const { t } = useTranslation(['common']);
  const [viewMode, setViewMode] = useState<ViewMode>('code');
  const [isCopied, setIsCopied] = useState(false);
  const resolvedTheme = useResolvedTheme();

  const { formattedJson, parsedJson, isValidJson } = useMemo(() => {
    try {
      const parsed = JSON.parse(content);
      const formatted = JSON.stringify(parsed, null, 2);
      return { formattedJson: formatted, parsedJson: parsed, isValidJson: true };
    } catch {
      return { formattedJson: content, parsedJson: null, isValidJson: false };
    }
  }, [content]);

  const monacoTheme = resolvedTheme === 'dark' ? 'clipboard-dark' : 'clipboard-light';
  const jsonViewStyle = resolvedTheme === 'dark' ? darkStyles : defaultStyles;
  const contentHeight = 'clamp(360px, 52vh, 920px)';

  const handleCopy = async () => {
    try {
      await copyToClipboard(formattedJson);
      setIsCopied(true);
      setTimeout(() => setIsCopied(false), 2000);
    } catch (error) {
      console.error('复制失败:', error);
    }
  };

  const toggleViewMode = () => {
    setViewMode((prev) => (prev === 'tree' ? 'code' : 'tree'));
  };

  return (
    <div className="flex min-h-[320px] min-w-0 flex-1 flex-col overflow-hidden min-[1200px]:min-h-[420px]">
      <div className="flex flex-wrap items-center justify-between gap-3 border-b border-border/70 px-3 py-2.5 min-[1200px]:px-4 min-[1200px]:py-3">
        <div className="flex min-w-0 flex-wrap items-center gap-2">
          <Badge variant="secondary">{t('codeEditor.json')}</Badge>
          <Badge variant="outline" className="text-xs">
            {viewMode === 'tree' ? t('jsonView.treeView') : t('jsonView.codeView')}
          </Badge>
        </div>
        <div className="flex items-center gap-2">
          <Button
            onClick={toggleViewMode}
            size="sm"
            variant="outline"
            className="h-8 rounded-lg"
            title={viewMode === 'tree' ? t('jsonView.switchToCode') : t('jsonView.switchToTree')}
          >
            {viewMode === 'tree' ? <Braces className="h-4 w-4" /> : <List className="h-4 w-4" />}
          </Button>
          <Button onClick={handleCopy} size="sm" variant="outline" className="h-8 rounded-lg">
            <Copy className="mr-2 h-4 w-4" />
            {isCopied ? t('codeEditor.copied') : t('codeEditor.copy')}
          </Button>
        </div>
      </div>

      <div
        data-testid="json-content-shell"
        className="min-h-0 flex-1 overflow-hidden"
        style={{ height: contentHeight }}
      >
        {!isValidJson || parsedJson === null ? (
          <div className="flex h-full items-center justify-center overflow-auto p-4 text-sm text-muted-foreground">
            {t('jsonView.invalidJson')}
          </div>
        ) : viewMode === 'tree' ? (
          <div data-testid="json-tree-scroll-region" className="h-full overflow-auto p-4">
            <JsonView
              data={parsedJson}
              shouldExpandNode={(level) => level < 3}
              style={jsonViewStyle}
            />
          </div>
        ) : (
          <MonacoEditor
            key={`json-${resolvedTheme}`}
            height={contentHeight}
            language="json"
            value={formattedJson}
            theme={monacoTheme}
            beforeMount={(monacoInstance) => {
              defineMonacoThemes(monacoInstance);
            }}
            options={{
              readOnly: true,
              minimap: { enabled: false },
              scrollBeyondLastLine: false,
              wordWrap: 'on',
              fontSize: 13,
              lineNumbers: 'on',
              lineNumbersMinChars: 3,
              glyphMargin: false,
              lineDecorationsWidth: 10,
              renderWhitespace: 'selection',
              automaticLayout: true,
              padding: { top: 18, bottom: 20 },
              tabSize: 2,
              folding: true,
              foldingHighlight: true,
            }}
          />
        )}
      </div>
    </div>
  );
}
