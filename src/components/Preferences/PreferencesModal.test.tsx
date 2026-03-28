import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import type { ReactNode } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { PreferencesModal } from './PreferencesModal';
import { useConfigStore } from '../../stores/configStore';
import { useClipboardStore } from '../../stores/clipboardStore';
import { invoke } from '@tauri-apps/api/core';

vi.mock('../../stores/configStore', () => ({
  useConfigStore: vi.fn(),
}));

vi.mock('../../stores/clipboardStore', () => ({
  useClipboardStore: {
    getState: vi.fn(),
  },
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, options?: string | { defaultValue?: string; returnObjects?: boolean }) => {
      if (typeof options === 'object' && options?.returnObjects) {
        return [];
      }

      if (typeof options === 'string') {
        return options;
      }

      return options?.defaultValue ?? key;
    },
    i18n: {
      language: 'zh',
      changeLanguage: vi.fn().mockResolvedValue(undefined),
    },
  }),
}));

vi.mock('../../services/analytics', () => ({
  analytics: {
    isEnabled: vi.fn(() => false),
    setEnabled: vi.fn(),
  },
}));

vi.mock('../../i18n/config', () => ({
  getSystemLanguage: vi.fn(() => 'zh'),
}));

vi.mock('../theme-provider', () => ({
  useTheme: () => ({
    theme: 'light',
    setTheme: vi.fn(),
  }),
}));

vi.mock('./ShortcutRecorder', () => ({
  ShortcutRecorder: () => <div data-testid="shortcut-recorder" />,
}));

vi.mock('../LogViewer/LogViewer', () => ({
  LogViewer: () => <div data-testid="log-viewer" />,
}));

vi.mock('../ui/dialog', () => ({
  Dialog: ({ children }: { children: ReactNode }) => <div>{children}</div>,
  DialogContent: ({ children }: { children: ReactNode }) => <div>{children}</div>,
  DialogHeader: ({ children }: { children: ReactNode }) => <div>{children}</div>,
  DialogTitle: ({ children }: { children: ReactNode }) => <div>{children}</div>,
}));

vi.mock('../ui/tabs', () => ({
  Tabs: ({ children }: { children: ReactNode }) => <div>{children}</div>,
  TabsList: ({ children }: { children: ReactNode }) => <div>{children}</div>,
  TabsTrigger: ({ children }: { children: ReactNode }) => <button>{children}</button>,
  TabsContent: ({ children }: { children: ReactNode }) => <div>{children}</div>,
}));

const mockedUseConfigStore = vi.mocked(useConfigStore);
const mockedInvoke = vi.mocked(invoke);
const mockedClipboardStoreGetState = vi.mocked(useClipboardStore.getState);

const baseConfig = {
  text: {
    max_size_mb: 1,
    expiry: 'Never' as const,
  },
  image: {
    expiry: 'Never' as const,
  },
  excluded_apps_v2: [],
  global_shortcut: 'CmdOrCtrl+Shift+V',
  auto_startup: false,
  auto_update: true,
  language: 'system',
};

let loadCacheStatistics: ReturnType<typeof vi.fn>;
let fetchHistory: ReturnType<typeof vi.fn>;
let invalidatePreview: ReturnType<typeof vi.fn>;

describe('PreferencesModal rebuild entry analysis action', () => {
  beforeEach(() => {
    vi.clearAllMocks();

    loadCacheStatistics = vi.fn().mockResolvedValue(undefined);
    fetchHistory = vi.fn().mockResolvedValue(undefined);
    invalidatePreview = vi.fn();

    mockedUseConfigStore.mockReturnValue({
      config: baseConfig,
      cacheStats: {
        db_size_bytes: 1024,
        images_size_bytes: 2048,
        total_entries: 12,
        text_entries: 10,
        image_entries: 2,
      },
      loading: false,
      error: null,
      showPreferences: true,
      cacheStatsLoading: false,
      cacheStatsError: null,
      loadConfig: vi.fn(),
      updateConfig: vi.fn(),
      loadCacheStatistics,
      registerGlobalShortcut: vi.fn(),
      setAutoStartup: vi.fn(),
      getAutoStartupStatus: vi.fn().mockResolvedValue(false),
      setShowPreferences: vi.fn(),
      formatBytes: vi.fn((value: number) => `${value} B`),
      getExpiryDisplayValue: vi.fn(() => 'never'),
      createExpiryOption: vi.fn(() => 'Never'),
    });

    mockedClipboardStoreGetState.mockReturnValue({
      fetchHistory,
      invalidatePreview,
    });

    mockedInvoke.mockImplementation(async (command) => {
      if (command === 'get_installed_applications') {
        return [];
      }

      throw new Error(`Unexpected invoke: ${command}`);
    });
  });

  it('点击 rebuild 会调用命令并刷新历史、预览缓存和统计', async () => {
    mockedInvoke.mockImplementation(async (command) => {
      if (command === 'get_installed_applications') {
        return [];
      }

      if (command === 'rebuild_entry_analysis') {
        return {
          scanned: 5,
          updated: 4,
          skipped: 1,
          failed: 0,
        };
      }

      throw new Error(`Unexpected invoke: ${command}`);
    });

    render(<PreferencesModal />);

    await waitFor(() => expect(loadCacheStatistics).toHaveBeenCalledTimes(1));
    loadCacheStatistics.mockClear();

    fireEvent.click(screen.getByTestId('analysis-rebuild-button'));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith('rebuild_entry_analysis', {
        batchSize: 250,
      });
    });

    await waitFor(() => {
      expect(invalidatePreview).toHaveBeenCalledTimes(1);
      expect(fetchHistory).toHaveBeenCalledTimes(1);
      expect(loadCacheStatistics).toHaveBeenCalledTimes(1);
    });

    expect(screen.getByTestId('analysis-rebuild-result')).toHaveTextContent(
      '已扫描 5 条，更新 4 条，跳过 1 条，失败 0 条。'
    );
  });

  it('命令失败时显示错误且不触发刷新副作用', async () => {
    mockedInvoke.mockImplementation(async (command) => {
      if (command === 'get_installed_applications') {
        return [];
      }

      if (command === 'rebuild_entry_analysis') {
        throw new Error('rebuild failed');
      }

      throw new Error(`Unexpected invoke: ${command}`);
    });

    render(<PreferencesModal />);

    await waitFor(() => expect(loadCacheStatistics).toHaveBeenCalledTimes(1));
    loadCacheStatistics.mockClear();

    fireEvent.click(screen.getByTestId('analysis-rebuild-button'));

    await waitFor(() => {
      expect(screen.getByTestId('analysis-rebuild-error')).toHaveTextContent('rebuild failed');
    });

    expect(invalidatePreview).not.toHaveBeenCalled();
    expect(fetchHistory).not.toHaveBeenCalled();
    expect(loadCacheStatistics).not.toHaveBeenCalled();
  });
});
