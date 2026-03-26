import { useEffect, useMemo, useState } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { BarChart3, Pause, Play, Trash2, View } from 'lucide-react';
import { getSystemLanguage } from './i18n/config';
import { ThemeProvider } from './components/theme-provider';
import { SettingsButton } from './components/settings-button';
import { PreferencesModal } from './components/Preferences/PreferencesModal';
import { MainLayout } from './components/Layout/MainLayout';
import { SearchBar } from './components/SearchBar/SearchBar';
import { ClipboardList } from './components/ClipboardList/ClipboardList';
import { DetailView } from './components/DetailView/DetailView';
import { MenuEventHandler } from './components/MenuEventHandler/MenuEventHandler';
import { UpdateChecker } from './components/UpdateChecker/UpdateChecker';
import { ClipboardMenuHandler } from './components/ClipboardMenuHandler';
import { useClipboardStore } from './stores/clipboardStore';
import { useConfigStore } from './stores/configStore';
import { analytics, ANALYTICS_EVENTS } from './services/analytics';
import { Button } from './components/ui/button';
import { StatisticsModal } from './components/Statistics/StatisticsModal';
import { TypeFilter } from './components/TypeFilter/TypeFilter';
import { cn } from './lib/utils';

const queryClient = new QueryClient();

function AppContent() {
  const { t, i18n } = useTranslation(['common', 'clipboard']);
  const {
    entries,
    isMonitoring,
    startMonitoring,
    stopMonitoring,
    clearHistory,
    fetchStatistics,
    setupEventListener,
  } = useClipboardStore();
  const { loadConfig } = useConfigStore();
  const [showStatistics, setShowStatistics] = useState(false);

  const updateWindowTitle = async (language: string) => {
    try {
      const title = language === 'zh' ? 'Dance' : 'Dance';
      await invoke('set_window_title', { title });
    } catch (error) {
      console.error('Failed to update window title:', error);
    }
  };

  useEffect(() => {
    const startTime = Date.now();
    analytics.track(ANALYTICS_EVENTS.APP_OPENED);

    setupEventListener();
    startMonitoring();

    loadConfig().then(async () => {
      const savedConfig = useConfigStore.getState().config;
      if (savedConfig?.language) {
        const targetLanguage =
          savedConfig.language === 'system' ? getSystemLanguage() : savedConfig.language;
        await i18n.changeLanguage(targetLanguage);
        await updateWindowTitle(targetLanguage);
      } else {
        await updateWindowTitle(i18n.language);
      }
    });

    analytics.trackPerformance(ANALYTICS_EVENTS.STARTUP_TIME, Date.now() - startTime);
  }, [setupEventListener, startMonitoring, loadConfig, i18n]);

  useEffect(() => {
    const handleLanguageChange = async (language: string) => {
      await updateWindowTitle(language);
    };

    i18n.on('languageChanged', handleLanguageChange);
    return () => {
      i18n.off('languageChanged', handleLanguageChange);
    };
  }, [i18n]);

  const favoriteCount = useMemo(
    () => entries.filter((entry) => entry.is_favorite).length,
    [entries]
  );
  const sourceCount = useMemo(
    () => new Set(entries.map((entry) => entry.source_app).filter(Boolean)).size,
    [entries]
  );

  const handleToggleMonitoring = async () => {
    if (isMonitoring) {
      await stopMonitoring();
    } else {
      await startMonitoring();
    }
  };

  const handleShowStatistics = async () => {
    await fetchStatistics();
    setShowStatistics(true);
  };

  const handleClearHistory = async () => {
    if (window.confirm(t('clipboard:actions.clearConfirmMessage'))) {
      await clearHistory();
    }
  };

  const toolbarButtonClass =
    'h-9 w-9 rounded-xl border-border/70 bg-background/75 text-foreground shadow-[0_8px_24px_rgba(15,23,42,0.08)] backdrop-blur-xl hover:bg-accent';

  return (
    <MainLayout>
      <MenuEventHandler />
      <UpdateChecker />
      <ClipboardMenuHandler />
      <PreferencesModal />
      <StatisticsModal isOpen={showStatistics} onClose={() => setShowStatistics(false)} />

      <div className="min-h-screen p-2.5 ">
        <a
          href="#clipboard-workspace"
          className="sr-only focus:not-sr-only focus:absolute focus:left-4 focus:top-4 focus:z-50 focus:rounded-xl focus:bg-background focus:px-4 focus:py-2 focus:text-sm focus:shadow-lg"
        >
          跳转到主内容
        </a>

        <div className="flex h-[calc(100vh-20px)] flex-col gap-3 overflow-hidden rounded-[24px] border border-border/70 bg-[linear-gradient(180deg,rgba(255,255,255,0.84),rgba(255,255,255,0.62))] p-3 shadow-[0_24px_80px_rgba(15,23,42,0.14)] backdrop-blur-2xl dark:bg-[linear-gradient(180deg,rgba(10,20,23,0.94),rgba(10,20,23,0.82))] min-[720px]:h-[calc(100vh-24px)] min-[1200px]:gap-4 min-[1200px]:rounded-[28px] min-[1200px]:p-4 md:p-5">
          <div className="flex flex-col gap-2.5 rounded-[20px] border border-border/70 bg-background/55 px-3 py-2.5 shadow-[inset_0_1px_0_rgba(255,255,255,0.45)] backdrop-blur-xl min-[1200px]:gap-3 min-[1200px]:rounded-[24px] min-[1200px]:py-3">
            <div className="flex flex-wrap items-center gap-2.5">
              <div className="flex min-w-0 items-center gap-3">
                <div className="flex h-9 w-9 items-center justify-center rounded-xl border border-primary/20 bg-primary/10 text-primary min-[1200px]:h-10 min-[1200px]:w-10 min-[1200px]:rounded-2xl">
                  <View className="h-4 w-4" />
                </div>
                <div className="min-w-0">
                  <div className="text-[10px] font-semibold uppercase tracking-[0.28em] text-primary/80 min-[1200px]:text-[11px] min-[1200px]:tracking-[0.32em]">
                    Dance
                  </div>
                  <div className="truncate text-sm font-semibold text-foreground">
                    {t('appTitle')}
                  </div>
                </div>
              </div>

              <div className="ml-auto flex flex-wrap items-center gap-1.5 min-[1200px]:gap-2">
                <Button
                  type="button"
                  variant="outline"
                  onClick={handleShowStatistics}
                  aria-label={t('clipboard:actions.viewStatistics')}
                  className={toolbarButtonClass}
                >
                  <BarChart3 className="h-4 w-4" />
                </Button>

                <Button
                  type="button"
                  variant={isMonitoring ? 'secondary' : 'default'}
                  onClick={handleToggleMonitoring}
                  className="h-9 rounded-xl border border-border/70 px-3 shadow-[0_8px_24px_rgba(15,23,42,0.08)] min-[1200px]:h-10 min-[1200px]:rounded-2xl min-[1200px]:px-4"
                >
                  {isMonitoring ? (
                    <Pause className="mr-2 h-4 w-4" />
                  ) : (
                    <Play className="mr-2 h-4 w-4" />
                  )}
                  {isMonitoring
                    ? t('clipboard:actions.stopMonitoring')
                    : t('clipboard:actions.startMonitoring')}
                </Button>

                <Button
                  type="button"
                  variant="outline"
                  onClick={handleClearHistory}
                  aria-label={t('clipboard:actions.clearHistory')}
                  className={toolbarButtonClass}
                >
                  <Trash2 className="h-4 w-4" />
                </Button>

                <SettingsButton
                  variant="outline"
                  buttonClassName={cn(toolbarButtonClass, 'text-muted-foreground')}
                />
              </div>
            </div>

            <div className="flex flex-wrap items-center gap-2.5 border-t border-border/60 pt-2.5 min-[1200px]:pt-3">
              <div className="flex min-w-0 items-center gap-2.5">
                <h2 className="truncate text-base font-semibold tracking-tight text-foreground min-[1200px]:text-lg">
                  查看剪切板历史
                </h2>

                <div className="hidden items-center gap-1.5 text-[11px] text-muted-foreground min-[860px]:flex min-[1200px]:gap-2 min-[1200px]:text-xs">
                  <span className="inline-flex items-center gap-1.5 rounded-full border border-border/70 bg-background/70 px-2.5 py-1">
                    <span className="font-semibold text-foreground">{entries.length}</span>
                    {t('all')}
                  </span>
                  <span className="inline-flex items-center gap-1.5 rounded-full border border-border/70 bg-background/70 px-2.5 py-1">
                    <span className="font-semibold text-foreground">{favoriteCount}</span>
                    {t('clipboard:actions.favorite')}
                  </span>
                  <span className="hidden items-center gap-1.5 rounded-full border border-border/70 bg-background/70 px-2.5 py-1 min-[1060px]:inline-flex">
                    <span className="font-semibold text-foreground">{sourceCount}</span>
                    {t('from')}
                  </span>
                </div>
              </div>

              <div className="ml-auto flex min-w-0 flex-1 items-center gap-2 min-[840px]:max-w-[560px]">
                <SearchBar compact className="min-w-0 flex-1" />
                <TypeFilter compact className="w-[168px] shrink-0" />
              </div>
            </div>
          </div>

          <div
            id="clipboard-workspace"
            className="grid min-h-0 flex-1 grid-cols-1 gap-3 overflow-hidden min-[840px]:grid-cols-[minmax(280px,320px)_minmax(0,1fr)] min-[1200px]:gap-4 min-[1280px]:grid-cols-[minmax(320px,380px)_minmax(0,1fr)]"
          >
            <aside className="flex min-h-0 flex-col gap-3 overflow-hidden">
              <div className="min-h-0 flex-1 overflow-hidden">
                <ClipboardList />
              </div>
            </aside>

            <section className="min-h-0 overflow-hidden">
              <DetailView />
            </section>
          </div>
        </div>
      </div>
    </MainLayout>
  );
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider defaultTheme="system" storageKey="clipboard-app-theme">
        <AppContent />
      </ThemeProvider>
    </QueryClientProvider>
  );
}

export default App;
