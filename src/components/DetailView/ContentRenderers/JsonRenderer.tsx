import { useState, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { Copy, Braces, List } from 'lucide-react';
import { JsonView, darkStyles, defaultStyles } from 'react-json-view-lite';
import 'react-json-view-lite/dist/index.css';
import { Button } from '../../ui/button';
import { Badge } from '../../ui/badge';
import { Card, CardContent, CardHeader } from '../../ui/card';
import { useResolvedTheme } from '../../../hooks/useResolvedTheme';
import { defineMonacoThemes } from '../../../utils/monacoTheme';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import MonacoEditor, { loader } from '@monaco-editor/react';
import * as monaco from 'monaco-editor';

loader.config({ monaco });

interface JsonRendererProps {
  content: string;
}

type ViewMode = 'tree' | 'code';

export function JsonRenderer({ content }: JsonRendererProps) {
  const { t } = useTranslation(['common']);
  const [viewMode, setViewMode] = useState<ViewMode>('tree');
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

  const handleCopy = async () => {
    try {
      await writeText(formattedJson);
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
    <div className="h-full flex flex-col">
      <Card className="flex-1 flex flex-col">
        <CardHeader className="pb-3 flex-shrink-0">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
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
                title={
                  viewMode === 'tree' ? t('jsonView.switchToCode') : t('jsonView.switchToTree')
                }
              >
                {viewMode === 'tree' ? (
                  <Braces className="w-4 h-4" />
                ) : (
                  <List className="w-4 h-4" />
                )}
              </Button>
              <Button onClick={handleCopy} size="sm" variant="outline">
                <Copy className="w-4 h-4 mr-2" />
                {isCopied ? t('codeEditor.copied') : t('codeEditor.copy')}
              </Button>
            </div>
          </div>
        </CardHeader>

        <CardContent className="p-0 flex-1 flex flex-col overflow-hidden">
          <div className="border-t flex-1 overflow-auto">
            {viewMode === 'tree' ? (
              isValidJson && parsedJson !== null ? (
                <div className="p-4">
                  <JsonView
                    data={parsedJson}
                    shouldExpandNode={(level) => level < 3}
                    style={jsonViewStyle}
                  />
                </div>
              ) : (
                <div className="p-4 text-muted-foreground">{t('jsonView.invalidJson')}</div>
              )
            ) : (
              <MonacoEditor
                key={`json-${resolvedTheme}`}
                height="100%"
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
                  renderWhitespace: 'selection',
                  automaticLayout: true,
                  padding: { top: 16, bottom: 16 },
                  tabSize: 2,
                  folding: true,
                  foldingHighlight: true,
                }}
              />
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
