import { Suspense, lazy, useEffect, useState } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { listen } from '@tauri-apps/api/event';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { View } from 'lucide-react';
import { getSystemLanguage } from './i18n/config';
import { ThemeProvider } from './components/theme-provider';
import { SettingsButton } from './components/settings-button';
import { MainLayout } from './components/Layout/MainLayout';
import { SearchBar } from './components/SearchBar/SearchBar';
import { ClipboardList } from './components/ClipboardList/ClipboardList';
import { MenuEventHandler } from './components/MenuEventHandler/MenuEventHandler';
import { ClipboardMenuHandler } from './components/ClipboardMenuHandler';
import { useClipboardStore } from './stores/clipboardStore';
import { useConfigStore } from './stores/configStore';
import { analytics, ANALYTICS_EVENTS } from './services/analytics';
import { TypeFilter } from './components/TypeFilter/TypeFilter';
import { cn } from './lib/utils';
import type { Statistics } from './types/clipboard';

const queryClient = new QueryClient();
const PreferencesModal = lazy(() =>
  import('./components/Preferences/PreferencesModal').then((mod) => ({
    default: mod.PreferencesModal,
  }))
);
const StatisticsModal = lazy(() =>
  import('./components/Statistics/StatisticsModal').then((mod) => ({
    default: mod.StatisticsModal,
  }))
);
const DetailView = lazy(() =>
  import('./components/DetailView/DetailView').then((mod) => ({
    default: mod.DetailView,
  }))
);
const UpdateChecker = lazy(() =>
  import('./components/UpdateChecker/UpdateChecker').then((mod) => ({
    default: mod.UpdateChecker,
  }))
);

function AppContent() {
  const { t, i18n } = useTranslation(['common', 'clipboard']);
  const { startMonitoring, setupEventListener } = useClipboardStore();
  const { loadConfig } = useConfigStore();
  const [showStatistics, setShowStatistics] = useState(false);
  const [modalStatistics, setModalStatistics] = useState<Statistics | null>(null);
  const isMacOS =
    typeof navigator !== 'undefined' && /(Mac|iPhone|iPod|iPad)/i.test(navigator.userAgent);

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

  useEffect(() => {
    let cleanup: (() => void) | undefined;

    void (async () => {
      const unlistenStats = await listen<Statistics>('show_statistics', (event) => {
        setModalStatistics(event.payload);
        setShowStatistics(true);
      });

      cleanup = () => {
        unlistenStats();
      };
    })();

    return () => {
      cleanup?.();
    };
  }, []);

  const toolbarButtonClass =
    'h-7 w-7 rounded-[9px] border-border/70 bg-background/78 text-foreground shadow-[0_6px_16px_rgba(15,23,42,0.05)] backdrop-blur-xl hover:bg-accent';
  const contentPaddingClassName = isMacOS ? 'min-h-screen px-3 pb-3 pt-0' : 'min-h-screen p-1.5';
  const shellClassName = isMacOS
    ? 'flex h-[calc(100vh-12px)] flex-col gap-1.5'
    : 'flex h-[calc(100vh-12px)] flex-col gap-2 overflow-hidden rounded-[20px] border border-border/70 bg-[linear-gradient(180deg,rgba(255,255,255,0.84),rgba(255,255,255,0.66))] p-2 shadow-[0_18px_48px_rgba(15,23,42,0.11)] backdrop-blur-2xl dark:bg-[linear-gradient(180deg,rgba(10,20,23,0.94),rgba(10,20,23,0.86))] min-[720px]:h-[calc(100vh-12px)] min-[1200px]:gap-2.5 min-[1200px]:rounded-[22px] min-[1200px]:p-2.5 md:p-3';
  const toolbarCardClassName = isMacOS
    ? 'rounded-[18px] border border-border/60 bg-background/50 px-2.5 py-1.5 shadow-[0_16px_40px_rgba(15,23,42,0.10)] backdrop-blur-xl'
    : 'rounded-[16px] border border-border/70 bg-background/60 px-1.5 py-1.5 shadow-[inset_0_1px_0_rgba(255,255,255,0.45)] backdrop-blur-xl min-[1200px]:rounded-[18px] min-[1200px]:px-2 min-[1200px]:py-1.5';

  const startWindowDrag = async () => {
    try {
      await getCurrentWindow().startDragging();
    } catch (error) {
      console.error('Failed to start window dragging:', error);
    }
  };

  const handleWindowDragMouseDown = (event: React.MouseEvent<HTMLDivElement>) => {
    if (!isMacOS || event.button !== 0) {
      return;
    }

    event.preventDefault();
    void startWindowDrag();
  };

  const handleToolbarMouseDown = (event: React.MouseEvent<HTMLDivElement>) => {
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

  return (
    <MainLayout>
      <MenuEventHandler />
      <Suspense fallback={null}>
        <UpdateChecker />
      </Suspense>
      <ClipboardMenuHandler />
      <Suspense fallback={null}>
        <PreferencesModal />
      </Suspense>
      {showStatistics ? (
        <Suspense fallback={null}>
          <StatisticsModal
            isOpen={showStatistics}
            onClose={() => {
              setShowStatistics(false);
              setModalStatistics(null);
            }}
            statistics={modalStatistics}
          />
        </Suspense>
      ) : null}

      <div className={contentPaddingClassName}>
        <a
          href="#clipboard-workspace"
          className="sr-only focus:not-sr-only focus:absolute focus:left-4 focus:top-4 focus:z-50 focus:rounded-xl focus:bg-background focus:px-4 focus:py-2 focus:text-sm focus:shadow-lg"
        >
          跳转到主内容
        </a>

        <div className={shellClassName}>
          {isMacOS ? (
            <div className="flex h-7 shrink-0 items-end px-1">
              <div
                aria-hidden="true"
                className="h-full w-[84px] shrink-0"
                onMouseDown={handleWindowDragMouseDown}
              />
              <div
                aria-hidden="true"
                className="h-full flex-1"
                onMouseDown={handleWindowDragMouseDown}
              />
              <div
                aria-hidden="true"
                className="h-full w-12 shrink-0"
                onMouseDown={handleWindowDragMouseDown}
              />
            </div>
          ) : null}

          <div className={toolbarCardClassName} onMouseDown={handleToolbarMouseDown}>
            {isMacOS ? (
              <div className="mb-1 grid grid-cols-[84px_minmax(0,1fr)_48px] gap-2">
                <div aria-hidden="true" className="h-1" onMouseDown={handleWindowDragMouseDown} />
                <div aria-hidden="true" className="h-1" onMouseDown={handleWindowDragMouseDown} />
                <div aria-hidden="true" className="h-1" onMouseDown={handleWindowDragMouseDown} />
              </div>
            ) : null}

            <div className="grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-1.5 min-[1200px]:gap-2">
              <div className="flex h-7 w-7 items-center justify-center rounded-[9px] border border-primary/20 bg-primary/10 text-primary min-[1200px]:h-[30px] min-[1200px]:w-[30px] min-[1200px]:rounded-[10px]">
                <View className="h-[15px] w-[15px]" />
              </div>

              <div className="mx-auto flex min-w-0 w-full max-w-[760px] items-center gap-1">
                <SearchBar compact className="min-w-0 flex-1" />
                <TypeFilter compact className="w-[132px] shrink-0 min-[1200px]:w-[146px]" />
              </div>

              <div className="flex items-center justify-end">
                <SettingsButton
                  variant="outline"
                  buttonClassName={cn(toolbarButtonClass, 'text-muted-foreground')}
                />
              </div>
            </div>
          </div>

          <div
            id="clipboard-workspace"
            className="grid min-h-0 flex-1 grid-cols-1 gap-2 overflow-hidden min-[840px]:grid-cols-[minmax(264px,320px)_minmax(0,1fr)] min-[1200px]:gap-2.5 min-[1280px]:grid-cols-[minmax(288px,348px)_minmax(0,1fr)]"
          >
            <aside className="flex min-h-0 flex-col gap-2 overflow-hidden">
              <div className="min-h-0 flex-1 overflow-hidden">
                <ClipboardList />
              </div>
            </aside>

            <section className="min-h-0 overflow-hidden">
              <Suspense
                fallback={
                  <div className="flex h-full min-h-[220px] items-center justify-center rounded-[18px] border border-border/70 bg-card/88">
                    <span className="text-sm text-muted-foreground">{t('detail.loading')}</span>
                  </div>
                }
              >
                <DetailView />
              </Suspense>
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
