import { useEffect, useState, useCallback, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '../ui/dialog';
import { Button } from '../ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select';
import { Input } from '../ui/input';
import { ScrollArea } from '../ui/scroll-area';
import { Card } from '../ui/card';
import { Badge } from '../ui/badge';
import { FileText, Trash2, Copy, Search, Settings, RefreshCw } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import * as Toast from '@radix-ui/react-toast';
import { copyToClipboard } from '../../stores/clipboardStore';

interface LogViewerProps {
  isOpen: boolean;
  onClose: () => void;
}

interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
  module?: string;
}

const LOG_LEVEL_COLORS = {
  error:
    'bg-red-100 text-red-800 border-red-200 dark:bg-red-900/20 dark:text-red-200 dark:border-red-800/50',
  warn: 'bg-yellow-100 text-yellow-800 border-yellow-200 dark:bg-yellow-900/20 dark:text-yellow-200 dark:border-yellow-800/50',
  info: 'bg-blue-100 text-blue-800 border-blue-200 dark:bg-blue-900/20 dark:text-blue-200 dark:border-blue-800/50',
  debug:
    'bg-green-100 text-green-800 border-green-200 dark:bg-green-900/20 dark:text-green-200 dark:border-green-800/50',
  trace:
    'bg-gray-100 text-gray-800 border-gray-200 dark:bg-gray-900/20 dark:text-gray-200 dark:border-gray-800/50',
};

export function LogViewer({ isOpen, onClose }: LogViewerProps) {
  const { t } = useTranslation(['preferences', 'common']);

  const LOG_LEVELS = [
    { value: 'all', label: t('logs.levels.all', 'All Levels') },
    { value: 'error', label: t('logs.levels.error', 'Error') },
    { value: 'warn', label: t('logs.levels.warn', 'Warning') },
    { value: 'info', label: t('logs.levels.info', 'Info') },
    { value: 'debug', label: t('logs.levels.debug', 'Debug') },
    { value: 'trace', label: t('logs.levels.trace', 'Trace') },
  ];
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [filteredLogs, setFilteredLogs] = useState<LogEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [searchTerm, setSearchTerm] = useState('');
  const [levelFilter, setLevelFilter] = useState('all');
  const [currentLogLevel, setCurrentLogLevel] = useState('info');
  const [showToast, setShowToast] = useState(false);
  const [toastMessage, setToastMessage] = useState('');
  const [autoRefresh, setAutoRefresh] = useState(true);
  const scrollAreaRef = useRef<HTMLDivElement>(null);
  const refreshIntervalRef = useRef<number>();

  const showToastMessage = useCallback((message: string) => {
    setToastMessage(message);
    setShowToast(true);
    setTimeout(() => setShowToast(false), 3000);
  }, []);

  const parseLogContent = useCallback((content: string): LogEntry[] => {
    if (!content.trim()) return [];

    const lines = content.split('\n').filter((line) => line.trim());
    const parsedLogs = lines.map((line) => {
      // Parse Tauri log plugin format: [YYYY-MM-DD][HH:MM:SS][module][LEVEL] message
      const tauriLogMatch = line.match(
        /^\[(\d{4}-\d{2}-\d{2})\]\[(\d{2}:\d{2}:\d{2})\]\[([^\]]+)\]\[([^\]]+)\]\s+(.+)$/
      );

      if (tauriLogMatch) {
        const [, date, time, module, level, message] = tauriLogMatch;
        return {
          timestamp: `${date} ${time}`,
          level: level.toLowerCase(),
          message,
          module: module,
        };
      }

      // Parse standard log format: TIMESTAMP [LEVEL] MESSAGE
      const standardMatch = line.match(
        /^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})?)\s+\[([^\]]+)\]\s+(.+)$/
      );

      if (standardMatch) {
        const [, timestamp, level, message] = standardMatch;
        return {
          timestamp: new Date(timestamp).toLocaleString(),
          level: level.toLowerCase(),
          message,
        };
      }

      // Fallback for non-standard format
      return {
        timestamp: new Date().toLocaleString(),
        level: 'info',
        message: line,
      };
    });

    // 倒序排列，最新的日志在最上面
    return parsedLogs.reverse();
  }, []);

  const loadLogs = useCallback(async () => {
    setLoading(true);
    try {
      const content = await invoke<string>('get_log_content');
      const entries = parseLogContent(content);
      setLogs(entries);
    } catch (error) {
      console.error('Failed to load logs:', error);
      showToastMessage(t('logs.errors.loadFailed', 'Failed to load logs'));
    } finally {
      setLoading(false);
    }
  }, [parseLogContent, showToastMessage, t]);

  const clearLogs = useCallback(async () => {
    try {
      await invoke('clear_logs');
      setLogs([]);
      showToastMessage(t('logs.messages.logsCleared', 'Logs cleared successfully'));
    } catch (error) {
      console.error('Failed to clear logs:', error);
      showToastMessage(t('logs.errors.clearFailed', 'Failed to clear logs'));
    }
  }, [showToastMessage, t]);

  const copyLogs = useCallback(async () => {
    const logText = filteredLogs
      .map((log) => `${log.timestamp} [${log.level.toUpperCase()}] ${log.message}`)
      .join('\n');

    try {
      await copyToClipboard(logText);
      showToastMessage(t('logs.messages.logsCopied', 'Logs copied to clipboard'));
    } catch (error) {
      console.error('Failed to copy logs:', error);
      showToastMessage(t('logs.errors.copyFailed', 'Failed to copy logs'));
    }
  }, [filteredLogs, showToastMessage, t]);

  const setLogLevel = useCallback(
    async (level: string) => {
      try {
        await invoke('set_log_level', { level });
        setCurrentLogLevel(level);
        showToastMessage(
          t('logs.messages.logLevelSet', 'Log level set to {{level}}', {
            level: level.toUpperCase(),
          })
        );
      } catch (error) {
        console.error('Failed to set log level:', error);
        showToastMessage(t('logs.errors.setLevelFailed', 'Failed to set log level'));
      }
    },
    [showToastMessage, t]
  );

  const getCurrentLogLevel = useCallback(async () => {
    try {
      const level = await invoke<string>('get_current_log_level');
      setCurrentLogLevel(level);
    } catch (error) {
      console.error('Failed to get log level:', error);
    }
  }, []);

  // Filter logs based on search and level
  useEffect(() => {
    let filtered = logs;

    if (levelFilter !== 'all') {
      filtered = filtered.filter((log) => log.level === levelFilter);
    }

    if (searchTerm) {
      const search = searchTerm.toLowerCase();
      filtered = filtered.filter(
        (log) =>
          log.message.toLowerCase().includes(search) || log.timestamp.toLowerCase().includes(search)
      );
    }

    setFilteredLogs(filtered);
  }, [logs, searchTerm, levelFilter]);

  // Load data when dialog opens
  useEffect(() => {
    if (isOpen) {
      loadLogs();
      getCurrentLogLevel();
    }
  }, [isOpen, loadLogs, getCurrentLogLevel]);

  // Auto refresh functionality
  useEffect(() => {
    if (isOpen && autoRefresh) {
      refreshIntervalRef.current = window.setInterval(() => {
        loadLogs();
      }, 2000); // 每2秒刷新一次

      return () => {
        if (refreshIntervalRef.current) {
          window.clearInterval(refreshIntervalRef.current);
        }
      };
    } else {
      if (refreshIntervalRef.current) {
        window.clearInterval(refreshIntervalRef.current);
      }
    }
  }, [isOpen, autoRefresh, loadLogs]);

  // Auto scroll to top when new logs are added (since logs are in reverse order)
  useEffect(() => {
    if (scrollAreaRef.current) {
      const scrollArea = scrollAreaRef.current.querySelector('[data-radix-scroll-area-viewport]');
      if (scrollArea) {
        scrollArea.scrollTop = 0;
      }
    }
  }, [filteredLogs]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (refreshIntervalRef.current) {
        window.clearInterval(refreshIntervalRef.current);
      }
    };
  }, []);

  return (
    <>
      <Dialog open={isOpen} onOpenChange={onClose}>
        <DialogContent className="max-w-6xl max-h-[80vh] flex flex-col">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <FileText className="w-5 h-5" />
              {t('logs.title', 'Log Viewer')}
            </DialogTitle>
          </DialogHeader>

          {/* Controls */}
          <div className="flex flex-wrap gap-4 p-4 bg-gray-50 dark:bg-gray-900/50 rounded-lg">
            <div className="flex items-center gap-2">
              <Search className="w-4 h-4" />
              <Input
                placeholder={t('logs.searchPlaceholder', 'Search logs...')}
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="w-48"
              />
            </div>

            <Select value={levelFilter} onValueChange={setLevelFilter}>
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {LOG_LEVELS.map((level) => (
                  <SelectItem key={level.value} value={level.value}>
                    {level.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <div className="flex items-center gap-2">
              <Settings className="w-4 h-4" />
              <span className="text-sm">{t('logs.logLevel', 'Level')}:</span>
              <Select value={currentLogLevel} onValueChange={setLogLevel}>
                <SelectTrigger className="w-24">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {LOG_LEVELS.slice(1).map((level) => (
                    <SelectItem key={level.value} value={level.value}>
                      {level.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="flex gap-2 ml-auto">
              <Button
                variant={autoRefresh ? 'default' : 'outline'}
                size="sm"
                onClick={() => setAutoRefresh(!autoRefresh)}
                title={
                  autoRefresh
                    ? t('logs.autoRefreshOn', 'Auto refresh enabled')
                    : t('logs.autoRefreshOff', 'Auto refresh disabled')
                }
              >
                <RefreshCw className={`w-4 h-4 ${autoRefresh ? 'animate-spin' : ''}`} />
                {autoRefresh ? t('logs.autoOn', 'Auto') : t('logs.autoOff', 'Manual')}
              </Button>
              <Button variant="outline" size="sm" onClick={loadLogs} disabled={loading}>
                <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={copyLogs}
                disabled={!filteredLogs.length}
              >
                <Copy className="w-4 h-4" />
              </Button>
              <Button variant="outline" size="sm" onClick={clearLogs}>
                <Trash2 className="w-4 h-4" />
              </Button>
            </div>
          </div>

          {/* Log Content */}
          <ScrollArea className="flex-1 h-96" ref={scrollAreaRef}>
            <div className="space-y-2 p-4">
              {loading ? (
                <div className="flex items-center justify-center py-8">
                  <RefreshCw className="w-6 h-6 animate-spin" />
                  <span className="ml-2">{t('logs.loading', 'Loading logs...')}</span>
                </div>
              ) : filteredLogs.length === 0 ? (
                <div className="text-center py-8 text-muted-foreground">
                  {logs.length === 0
                    ? t('logs.noLogs', 'No logs available')
                    : t('logs.noMatches', 'No logs match your filters')}
                </div>
              ) : (
                filteredLogs.map((log, index) => (
                  <Card key={index} className="p-3">
                    <div className="flex items-start gap-3">
                      <Badge
                        variant="outline"
                        className={`shrink-0 ${LOG_LEVEL_COLORS[log.level as keyof typeof LOG_LEVEL_COLORS] || 'bg-gray-100'}`}
                      >
                        {log.level.toUpperCase()}
                      </Badge>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 text-xs text-muted-foreground mb-1">
                          <span>{log.timestamp}</span>
                          {log.module && (
                            <>
                              <span>•</span>
                              <code className="px-1 py-0.5 bg-muted rounded text-xs">
                                {log.module.split('::').pop()}
                              </code>
                            </>
                          )}
                        </div>
                        <div className="text-sm font-mono break-words">{log.message}</div>
                      </div>
                    </div>
                  </Card>
                ))
              )}
            </div>
          </ScrollArea>

          {/* Footer */}
          <div className="flex items-center justify-between pt-4 border-t">
            <div className="text-sm text-muted-foreground">
              {t('logs.entriesCount', '{{filtered}} of {{total}} entries', {
                filtered: filteredLogs.length,
                total: logs.length,
              })}
            </div>
            <Button onClick={onClose}>{t('common:close', 'Close')}</Button>
          </div>
        </DialogContent>
      </Dialog>

      <Toast.Provider swipeDirection="right">
        <Toast.Root
          className="bg-background border border-border rounded-lg p-4 shadow-lg"
          open={showToast}
          onOpenChange={setShowToast}
        >
          <Toast.Description className="text-sm">{toastMessage}</Toast.Description>
        </Toast.Root>
        <Toast.Viewport className="fixed bottom-0 right-0 flex flex-col p-4 gap-2 w-96 max-w-[100vw] m-0 list-none z-[2147483647] outline-none" />
      </Toast.Provider>
    </>
  );
}
