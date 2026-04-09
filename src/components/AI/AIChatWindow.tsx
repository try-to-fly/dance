import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useTranslation } from 'react-i18next';
import { Bot, ChevronDown, Loader2, Send, Settings2, Sparkles, Trash2 } from 'lucide-react';
import { Badge } from '../ui/badge';
import { Button } from '../ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { cn } from '../../lib/utils';
import { formatUnknownError } from '../../lib/errors';
import { useAiStore } from '../../stores/aiStore';
import { useConfigStore } from '../../stores/configStore';
import type { AiChatWindowPayload } from '../../types/ai';
import {
  AI_CHAT_WINDOW_NAVIGATE_EVENT,
  type AiChatWindowNavigationPayload,
  getAiChatWindowTokenFromLocation,
} from '../../lib/ai/chatWindow';

const CHAT_SUGGESTIONS = ['翻译成英文', '提取所有 URL', '总结重点', '整理成待办清单'];

const takeAiChatWindowPayload = (token: string) =>
  invoke<AiChatWindowPayload | null>('take_ai_chat_window_payload', { token });

export function AIChatWindow() {
  const { t } = useTranslation('common');
  const { config, setShowPreferences } = useConfigStore();
  const { activeSourceKey, sessions, initializeSession, setInput, sendPrompt, clearConversation } =
    useAiStore();
  const [hydrating, setHydrating] = useState(true);
  const [sourceExpanded, setSourceExpanded] = useState(false);
  const [focusTick, setFocusTick] = useState(0);
  const inputRef = useRef<HTMLTextAreaElement | null>(null);
  const session = activeSourceKey ? sessions[activeSourceKey] : undefined;
  const hasAiConfig = Boolean(config?.llm.api_key.trim() && config?.llm.model.trim());
  const isMacOS =
    typeof navigator !== 'undefined' && /(Mac|iPhone|iPod|iPad)/i.test(navigator.userAgent);

  const applyPayload = (payload: AiChatWindowPayload) => {
    initializeSession({
      ...payload,
      mode: 'chat',
    });
    setSourceExpanded(false);
    setFocusTick((current) => current + 1);
    setHydrating(false);
  };

  const loadPayload = async (token: string | null) => {
    if (!token) {
      setHydrating(false);
      return;
    }

    setHydrating(true);

    try {
      const payload = await takeAiChatWindowPayload(token);
      if (payload) {
        applyPayload(payload);
        return;
      }
    } catch (error) {
      console.error('[AIChatWindow] 读取窗口初始化数据失败:', formatUnknownError(error));
    }

    setHydrating(false);
  };

  useEffect(() => {
    void loadPayload(getAiChatWindowTokenFromLocation());

    const currentWindow = getCurrentWebviewWindow();
    let unlisten: (() => void) | undefined;

    currentWindow
      .listen<AiChatWindowNavigationPayload>(AI_CHAT_WINDOW_NAVIGATE_EVENT, (event) => {
        void loadPayload(event.payload.token);
      })
      .then((cleanup) => {
        unlisten = cleanup;
      })
      .catch((error) => {
        console.error('[AIChatWindow] 监听窗口切换事件失败:', formatUnknownError(error));
      });

    return () => {
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    if (!activeSourceKey || !hasAiConfig) {
      return;
    }

    const timer = window.setTimeout(() => {
      inputRef.current?.focus();
      const inputLength = inputRef.current?.value.length ?? 0;
      inputRef.current?.setSelectionRange(inputLength, inputLength);
    }, 0);

    return () => window.clearTimeout(timer);
  }, [activeSourceKey, focusTick, hasAiConfig]);

  useEffect(() => {
    const nextTitle = session?.title
      ? `${t('detail.ai.chatTitle')} · ${session.title}`
      : t('detail.ai.chatTitle');

    void getCurrentWindow()
      .setTitle(nextTitle)
      .catch((error) => {
        console.error('[AIChatWindow] 更新窗口标题失败:', formatUnknownError(error));
      });
  }, [session?.title, t]);

  const startWindowDrag = async () => {
    try {
      await getCurrentWindow().startDragging();
    } catch (error) {
      console.error('[AIChatWindow] 开始拖拽窗口失败:', formatUnknownError(error));
    }
  };

  const handleHeaderMouseDown = (event: React.MouseEvent<HTMLDivElement>) => {
    if (!isMacOS || event.button !== 0) {
      return;
    }

    const target = event.target;
    if (
      target instanceof Element &&
      target.closest(
        'button, input, select, textarea, a, [role="button"], [data-no-window-drag="true"]'
      )
    ) {
      return;
    }

    event.preventDefault();
    void startWindowDrag();
  };

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
          <Button type="button" onClick={() => setShowPreferences(true)} className="rounded-xl">
            {t('detail.ai.openSettings')}
          </Button>
        </CardContent>
      </Card>
    </div>
  );

  const renderSourceStrip = () => {
    if (!session) {
      return null;
    }

    return (
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
              <pre className="max-h-[128px] overflow-y-auto whitespace-pre-wrap break-words font-mono text-xs leading-5 text-foreground">
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
  };

  const renderChatWorkspace = () => {
    if (!session) {
      return null;
    }

    return (
      <Card className="flex h-full min-h-0 flex-1 flex-col overflow-hidden rounded-[22px] border-border/70 bg-card/92 shadow-[0_20px_60px_rgba(15,23,42,0.10)]">
        <CardHeader className="border-b border-border/70 bg-[linear-gradient(135deg,rgba(59,130,246,0.10),rgba(255,255,255,0.64))] px-3 py-2.5 dark:bg-[linear-gradient(135deg,rgba(59,130,246,0.12),rgba(15,23,42,0.84))] sm:px-4">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div className="flex min-w-0 items-center gap-2">
              <CardTitle className="truncate text-sm font-semibold tracking-tight">
                {t('detail.ai.chatTitle')}
              </CardTitle>
              {session.title ? (
                <Badge
                  variant="outline"
                  className="max-w-[220px] rounded-full px-2 py-0.5 text-[11px]"
                >
                  <span className="block truncate">{session.title}</span>
                </Badge>
              ) : null}
            </div>

            <div className="flex items-center gap-2">
              {config?.llm.model ? (
                <Badge variant="outline" className="rounded-full px-2 py-0.5 text-[11px]">
                  {config.llm.model}
                </Badge>
              ) : null}
              <Button
                type="button"
                variant="outline"
                size="icon"
                className="h-8 w-8 rounded-lg"
                onClick={clearConversation}
                aria-label={t('detail.ai.clearConversation')}
                title={t('detail.ai.clearConversation')}
              >
                <Trash2 className="h-3.5 w-3.5" />
              </Button>
            </div>
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
                    className={cn(
                      'flex',
                      message.role === 'user' ? 'justify-end' : 'justify-start'
                    )}
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
                ref={inputRef}
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
  };

  return (
    <div className="min-h-screen bg-[radial-gradient(circle_at_top,rgba(59,130,246,0.10),transparent_32%),linear-gradient(180deg,rgba(255,255,255,0.98),rgba(244,247,251,0.96))] p-3 dark:bg-[radial-gradient(circle_at_top,rgba(59,130,246,0.16),transparent_30%),linear-gradient(180deg,rgba(8,16,24,0.98),rgba(8,16,24,0.94))] sm:p-4">
      <div className="mx-auto flex h-[calc(100vh-24px)] max-w-6xl flex-col gap-3 sm:h-[calc(100vh-32px)]">
        <div
          className="overflow-hidden rounded-[22px] border border-border/70 bg-background/78 shadow-[0_18px_48px_rgba(15,23,42,0.10)] backdrop-blur-xl"
          onMouseDown={handleHeaderMouseDown}
        >
          {isMacOS ? (
            <div
              aria-hidden="true"
              className="h-6 border-b border-border/50 bg-[linear-gradient(180deg,rgba(255,255,255,0.42),rgba(255,255,255,0.12))] dark:bg-[linear-gradient(180deg,rgba(255,255,255,0.08),rgba(255,255,255,0.02))]"
            />
          ) : null}

          <div className="flex items-center justify-between px-4 py-3.5 sm:px-4 sm:py-4">
            <div className="flex min-w-0 items-center gap-3">
              <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-[14px] border border-primary/20 bg-primary/10 text-primary">
                <Bot className="h-5 w-5" />
              </div>
              <div className="min-w-0">
                <h1 className="truncate text-base font-semibold tracking-tight">
                  {t('detail.ai.chatTitle')}
                </h1>
                <p className="truncate text-xs text-muted-foreground">{t('detail.ai.chatHint')}</p>
              </div>
            </div>

            <Button
              type="button"
              variant="outline"
              size="icon"
              className="h-9 w-9 rounded-xl"
              data-no-window-drag="true"
              onClick={() => setShowPreferences(true)}
              aria-label={t('detail.ai.openSettings')}
              title={t('detail.ai.openSettings')}
            >
              <Settings2 className="h-4 w-4" />
            </Button>
          </div>
        </div>

        {hydrating ? (
          <div className="flex flex-1 items-center justify-center">
            <div className="flex items-center gap-2 rounded-full border border-border/70 bg-background/88 px-4 py-2 text-sm text-muted-foreground shadow-sm">
              <Loader2 className="h-4 w-4 animate-spin" />
              {t('detail.loading')}
            </div>
          </div>
        ) : !hasAiConfig ? (
          renderSetupState()
        ) : !session ? (
          <div className="flex flex-1 items-center justify-center">
            <Card className="w-full max-w-xl rounded-[24px] border-border/70 bg-card/92 shadow-[0_20px_60px_rgba(15,23,42,0.10)]">
              <CardContent className="p-6">
                <p className="text-sm leading-7 text-muted-foreground">
                  {t('detail.ai.chatEmpty')}
                </p>
              </CardContent>
            </Card>
          </div>
        ) : (
          <>
            {renderSourceStrip()}
            <div className="min-h-0 flex-1">{renderChatWorkspace()}</div>
          </>
        )}
      </div>
    </div>
  );
}
