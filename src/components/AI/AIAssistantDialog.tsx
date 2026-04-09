import { useCallback, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
  Bot,
  Check,
  ChevronDown,
  Copy,
  Languages,
  Loader2,
  MessageSquareText,
  Send,
  Settings2,
  Sparkles,
  Trash2,
} from 'lucide-react';
import { Badge } from '../ui/badge';
import { Button } from '../ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '../ui/dialog';
import { Tabs, TabsList, TabsTrigger } from '../ui/tabs';
import { cn } from '../../lib/utils';
import { openAiChatWindow } from '../../lib/ai/chatWindow';
import { formatUnknownError } from '../../lib/errors';
import { copyToClipboard } from '../../stores/clipboardStore';
import { useAiStore } from '../../stores/aiStore';
import { useConfigStore } from '../../stores/configStore';
import { type AiDialogMode } from '../../types/ai';

const CHAT_SUGGESTIONS = ['翻译成英文', '提取所有 URL', '总结重点', '整理成待办清单'];

export function AIAssistantDialog() {
  const { t } = useTranslation('common');
  const { config, setShowPreferences } = useConfigStore();
  const {
    isOpen,
    mode,
    activeSourceKey,
    sessions,
    closeDialog,
    setMode,
    setInput,
    sendPrompt,
    requestTranslation,
    clearConversation,
  } = useAiStore();
  const [copyFeedback, setCopyFeedback] = useState<'translation' | null>(null);
  const [sourceExpanded, setSourceExpanded] = useState(false);

  const session = activeSourceKey ? sessions[activeSourceKey] : undefined;
  const hasAiConfig = Boolean(config?.llm.api_key.trim() && config?.llm.model.trim());

  useEffect(() => {
    if (
      isOpen &&
      mode === 'translate' &&
      hasAiConfig &&
      session &&
      session.translation.status === 'idle'
    ) {
      void requestTranslation();
    }
  }, [hasAiConfig, isOpen, mode, requestTranslation, session]);

  useEffect(() => {
    if (!copyFeedback) {
      return;
    }

    const timer = window.setTimeout(() => setCopyFeedback(null), 1800);
    return () => window.clearTimeout(timer);
  }, [copyFeedback]);

  useEffect(() => {
    if (!isOpen) {
      setSourceExpanded(false);
    }
  }, [isOpen, activeSourceKey]);

  const handleOpenChatWindow = useCallback(async () => {
    if (!session) {
      return;
    }

    setMode('translate');

    try {
      await openAiChatWindow(
        {
          sourceKey: session.sourceKey,
          title: session.title,
          sourceText: session.sourceText,
        },
        {
          windowTitle: t('detail.ai.chatTitle'),
        }
      );
      closeDialog();
    } catch (error) {
      console.error('[AIAssistantDialog] 打开聊天窗口失败:', formatUnknownError(error));
    }
  }, [closeDialog, session, setMode, t]);

  useEffect(() => {
    if (isOpen && mode === 'chat') {
      void handleOpenChatWindow();
    }
  }, [handleOpenChatWindow, isOpen, mode]);

  if (!session) {
    return null;
  }

  const handleOpenPreferences = () => {
    setShowPreferences(true);
  };

  const handleModeChange = (nextMode: string) => {
    if (nextMode === 'chat') {
      void handleOpenChatWindow();
      return;
    }

    setMode(nextMode as AiDialogMode);
  };

  const handleCopyTranslation = async () => {
    if (!session.translation.content) {
      return;
    }

    await copyToClipboard(session.translation.content);
    setCopyFeedback('translation');
  };

  const translateActionLabel =
    session.translation.status === 'loading'
      ? t('detail.ai.translating')
      : t('detail.ai.retryTranslate');
  const copyTranslationLabel =
    copyFeedback === 'translation' ? t('detail.ai.copied') : t('detail.ai.copyResult');
  const continueChatLabel = t('detail.ai.continueChat');
  const clearConversationLabel = t('detail.ai.clearConversation');

  const renderSetupState = () => (
    <div className="flex flex-1 items-center justify-center p-4 sm:p-5">
      <Card className="w-full max-w-lg rounded-[24px] border-primary/15 bg-[linear-gradient(135deg,rgba(45,212,191,0.08),rgba(255,255,255,0.92))] shadow-[0_20px_50px_rgba(15,23,42,0.10)] dark:bg-[linear-gradient(135deg,rgba(45,212,191,0.12),rgba(15,23,42,0.92))]">
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-2 text-base">
            <Settings2 className="h-4 w-4 text-primary" />
            {t('detail.ai.setupTitle')}
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <p className="text-sm leading-6 text-muted-foreground">
            {t('detail.ai.setupDescription')}
          </p>
          <Button type="button" onClick={handleOpenPreferences} className="rounded-xl">
            {t('detail.ai.openSettings')}
          </Button>
        </CardContent>
      </Card>
    </div>
  );

  const renderSourceStrip = () => (
    <Card className="shrink-0 overflow-hidden rounded-[18px] border-border/70 bg-muted/20 shadow-none">
      <button
        type="button"
        className="flex w-full items-center gap-3 px-3 py-2.5 text-left transition-colors hover:bg-muted/30"
        onClick={() => setSourceExpanded((current) => !current)}
      >
        <div className="min-w-0 flex-1">
          <div className="flex min-w-0 items-center gap-2">
            <Sparkles className="h-4 w-4 shrink-0 text-primary" />
            <span className="text-xs font-medium text-foreground">
              {t('detail.ai.sourceTitle')}
            </span>
            {session.title ? (
              <Badge
                variant="secondary"
                className="max-w-full rounded-full px-2 py-0.5 text-[11px]"
              >
                <span className="block max-w-[180px] truncate">{session.title}</span>
              </Badge>
            ) : null}
          </div>
          <p className="mt-1 truncate text-xs text-muted-foreground">{session.sourceText}</p>
        </div>
        <ChevronDown
          className={cn(
            'h-4 w-4 shrink-0 text-muted-foreground transition-transform',
            sourceExpanded ? 'rotate-180' : ''
          )}
        />
      </button>

      {sourceExpanded ? (
        <div className="border-t border-border/70 px-3 pb-3 pt-2">
          <div className="rounded-[14px] border border-border/70 bg-background/80 p-3">
            <pre className="max-h-[104px] overflow-y-auto whitespace-pre-wrap break-words font-mono text-xs leading-5 text-foreground">
              {session.sourceText}
            </pre>
          </div>
          <p className="mt-2 text-[11px] leading-5 text-muted-foreground">
            {t('detail.ai.sourceHint')}
          </p>
        </div>
      ) : null}
    </Card>
  );

  const renderTranslateWorkspace = () => (
    <Card className="flex h-full min-h-0 flex-1 flex-col overflow-hidden rounded-[22px] border-border/70 bg-card/92 shadow-[0_20px_60px_rgba(15,23,42,0.10)]">
      <CardHeader className="border-b border-border/70 bg-[linear-gradient(135deg,rgba(45,212,191,0.10),rgba(255,255,255,0.64))] px-3 py-2.5 dark:bg-[linear-gradient(135deg,rgba(45,212,191,0.10),rgba(15,23,42,0.84))] sm:px-4">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <CardTitle className="text-sm font-semibold tracking-tight">
            {t('detail.ai.translateTitle')}
          </CardTitle>

          <div className="flex flex-wrap items-center gap-2">
            {session.translation.model ? (
              <Badge variant="outline" className="rounded-full px-2 py-0.5 text-[11px]">
                {session.translation.model}
              </Badge>
            ) : null}
            <Button
              type="button"
              variant="outline"
              size="icon"
              className="h-8 w-8 rounded-lg"
              onClick={() => void requestTranslation()}
              disabled={session.translation.status === 'loading'}
              aria-label={translateActionLabel}
              title={translateActionLabel}
            >
              {session.translation.status === 'loading' ? (
                <Loader2 className="h-3.5 w-3.5 animate-spin" />
              ) : (
                <Languages className="h-3.5 w-3.5" />
              )}
            </Button>
            <Button
              type="button"
              variant="secondary"
              size="icon"
              className="h-8 w-8 rounded-lg"
              onClick={() => void handleCopyTranslation()}
              disabled={!session.translation.content}
              aria-label={copyTranslationLabel}
              title={copyTranslationLabel}
            >
              {copyFeedback === 'translation' ? (
                <Check className="h-3.5 w-3.5" />
              ) : (
                <Copy className="h-3.5 w-3.5" />
              )}
            </Button>
            <Button
              type="button"
              size="icon"
              className="h-8 w-8 rounded-lg"
              onClick={() => void handleOpenChatWindow()}
              aria-label={continueChatLabel}
              title={continueChatLabel}
            >
              <MessageSquareText className="h-3.5 w-3.5" />
            </Button>
          </div>
        </div>
      </CardHeader>

      <CardContent className="flex min-h-0 flex-1 flex-col p-0">
        {session.translation.status === 'loading' ? (
          <div className="flex flex-1 items-center justify-center p-4">
            <div className="flex items-center gap-2 rounded-full border border-border/70 bg-background/80 px-3 py-1.5 text-sm text-muted-foreground">
              <Loader2 className="h-4 w-4 animate-spin" />
              {t('detail.ai.translating')}
            </div>
          </div>
        ) : session.translation.status === 'error' ? (
          <div className="p-4">
            <div className="rounded-[18px] border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm leading-6 text-destructive">
              {session.translation.error}
            </div>
          </div>
        ) : session.translation.content ? (
          <div className="min-h-0 flex-1 overflow-y-auto p-3 sm:p-4">
            <div className="rounded-[18px] border border-border/70 bg-[linear-gradient(180deg,rgba(45,212,191,0.10),rgba(255,255,255,0.88))] p-4 shadow-[inset_0_1px_0_rgba(255,255,255,0.65)] dark:bg-[linear-gradient(180deg,rgba(45,212,191,0.12),rgba(15,23,42,0.82))]">
              <pre className="whitespace-pre-wrap break-words text-sm leading-7 text-foreground">
                {session.translation.content}
              </pre>
            </div>
          </div>
        ) : (
          <div className="flex flex-1 items-center justify-center px-6 py-10 text-sm text-muted-foreground">
            {t('detail.ai.translateEmpty')}
          </div>
        )}
      </CardContent>
    </Card>
  );

  const renderChatWorkspace = () => (
    <Card className="flex h-full min-h-0 flex-1 flex-col overflow-hidden rounded-[22px] border-border/70 bg-card/92 shadow-[0_20px_60px_rgba(15,23,42,0.10)]">
      <CardHeader className="border-b border-border/70 bg-[linear-gradient(135deg,rgba(59,130,246,0.10),rgba(255,255,255,0.64))] px-3 py-2.5 dark:bg-[linear-gradient(135deg,rgba(59,130,246,0.12),rgba(15,23,42,0.84))] sm:px-4">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <CardTitle className="text-sm font-semibold tracking-tight">
            {t('detail.ai.chatTitle')}
          </CardTitle>
          <Button
            type="button"
            variant="outline"
            size="icon"
            className="h-8 w-8 rounded-lg"
            onClick={clearConversation}
            aria-label={clearConversationLabel}
            title={clearConversationLabel}
          >
            <Trash2 className="h-3.5 w-3.5" />
          </Button>
        </div>
      </CardHeader>

      <CardContent className="flex min-h-0 flex-1 flex-col p-0">
        <div className="min-h-0 flex-1 overflow-y-auto p-3 sm:p-4">
          {session.messages.length === 0 ? (
            <div className="flex h-full items-center justify-center">
              <div className="w-full max-w-2xl rounded-[18px] border border-dashed border-border/80 bg-muted/25 px-4 py-4">
                <p className="text-sm leading-6 text-muted-foreground">
                  {t('detail.ai.chatEmpty')}
                </p>
                <div className="mt-4 flex flex-wrap gap-2">
                  {CHAT_SUGGESTIONS.map((suggestion) => (
                    <Button
                      key={suggestion}
                      type="button"
                      variant="outline"
                      size="sm"
                      className="h-8 rounded-full px-3 text-xs"
                      onClick={() => void sendPrompt(suggestion)}
                    >
                      {suggestion}
                    </Button>
                  ))}
                </div>
              </div>
            </div>
          ) : (
            <div className="space-y-3">
              {session.messages.map((message) => (
                <div
                  key={message.id}
                  className={cn('flex', message.role === 'user' ? 'justify-end' : 'justify-start')}
                >
                  <div
                    className={cn(
                      'max-w-[92%] rounded-[18px] px-4 py-3 text-sm leading-7 shadow-sm',
                      message.role === 'user'
                        ? 'bg-primary text-primary-foreground'
                        : message.error
                          ? 'border border-destructive/30 bg-destructive/10 text-destructive'
                          : 'border border-border/70 bg-muted/35 text-foreground'
                    )}
                  >
                    <pre className="whitespace-pre-wrap break-words font-sans">
                      {message.content}
                    </pre>
                    <div className="mt-2 flex flex-wrap items-center justify-between gap-3 text-[11px] text-current/70">
                      <span>
                        {new Date(message.createdAt).toLocaleTimeString([], {
                          hour: '2-digit',
                          minute: '2-digit',
                        })}
                      </span>
                      {message.model ? <span>{message.model}</span> : null}
                    </div>
                  </div>
                </div>
              ))}

              {session.loading ? (
                <div className="flex justify-start">
                  <div className="rounded-[18px] border border-border/70 bg-muted/25 px-4 py-3 text-sm text-muted-foreground">
                    <div className="flex items-center gap-2">
                      <Loader2 className="h-4 w-4 animate-spin" />
                      {t('detail.ai.replying')}
                    </div>
                  </div>
                </div>
              ) : null}
            </div>
          )}
        </div>

        <div className="border-t border-border/70 p-3">
          <div className="flex items-end gap-2 rounded-[18px] border border-border/70 bg-background/92 p-2 shadow-[0_12px_28px_rgba(15,23,42,0.06)]">
            <textarea
              rows={2}
              value={session.input}
              onChange={(event) => setInput(event.target.value)}
              onKeyDown={(event) => {
                if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
                  event.preventDefault();
                  void sendPrompt();
                }
              }}
              placeholder={t('detail.ai.inputPlaceholder')}
              className="min-h-[56px] flex-1 resize-none border-0 bg-transparent px-1 py-1 text-sm leading-6 outline-none placeholder:text-muted-foreground"
            />

            <Button
              type="button"
              size="sm"
              className="h-9 rounded-lg px-3"
              onClick={() => void sendPrompt()}
              disabled={session.loading || !session.input.trim()}
            >
              {session.loading ? (
                <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
              ) : (
                <Send className="mr-1.5 h-3.5 w-3.5" />
              )}
              {t('detail.ai.send')}
            </Button>
          </div>
          <p className="mt-1.5 px-1 text-[11px] leading-4 text-muted-foreground">
            {t('detail.ai.inputHint')}
          </p>
        </div>
      </CardContent>
    </Card>
  );

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && closeDialog()}>
      <DialogContent className="flex h-[92vh] max-h-[92vh] max-w-6xl flex-col overflow-hidden border border-border/70 bg-background/96 p-0 shadow-[0_32px_96px_rgba(15,23,42,0.22)] backdrop-blur-2xl sm:rounded-[24px]">
        <DialogHeader className="border-b border-border/70 bg-[radial-gradient(circle_at_top_left,rgba(45,212,191,0.16),transparent_34%),radial-gradient(circle_at_top_right,rgba(251,191,36,0.10),transparent_26%)] px-4 py-3 sm:px-5">
          <div className="flex flex-wrap items-center gap-3 pr-10">
            <DialogTitle className="mr-auto flex items-center gap-2 text-base font-semibold tracking-tight">
              <Bot className="h-4 w-4 text-primary" />
              {t('detail.ai.dialogTitle')}
            </DialogTitle>

            {hasAiConfig ? (
              <Tabs value={mode} onValueChange={handleModeChange}>
                <TabsList className="h-9 rounded-xl bg-muted/70 p-1">
                  <TabsTrigger
                    value="translate"
                    className="h-7 rounded-lg px-3 text-xs data-[state=active]:bg-background data-[state=active]:shadow-sm"
                  >
                    <Languages className="mr-1.5 h-3.5 w-3.5" />
                    {t('detail.ai.translate')}
                  </TabsTrigger>
                  <TabsTrigger
                    value="chat"
                    className="h-7 rounded-lg px-3 text-xs data-[state=active]:bg-background data-[state=active]:shadow-sm"
                  >
                    <MessageSquareText className="mr-1.5 h-3.5 w-3.5" />
                    {t('detail.ai.chat')}
                  </TabsTrigger>
                </TabsList>
              </Tabs>
            ) : null}

            {config?.llm.model ? (
              <Badge variant="outline" className="rounded-full px-2.5 py-1 text-[11px]">
                {config.llm.model}
              </Badge>
            ) : null}
          </div>
        </DialogHeader>

        {!hasAiConfig ? (
          renderSetupState()
        ) : (
          <div className="flex min-h-0 flex-1 flex-col gap-2 p-3 sm:p-4">
            {renderSourceStrip()}
            <div className="flex min-h-0 flex-1">
              {mode === 'translate' ? renderTranslateWorkspace() : renderChatWorkspace()}
            </div>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
