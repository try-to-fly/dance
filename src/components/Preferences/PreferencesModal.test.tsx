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
  ShortcutRecorder: ({
    value,
    onChange,
  }: {
    value: string;
    onChange: (shortcut: string) => void;
  }) => (
    <div>
      <div data-testid="shortcut-value">{value}</div>
      <button data-testid="shortcut-recorder" onClick={() => onChange('CmdOrCtrl+Alt+C')}>
        更新快捷键
      </button>
    </div>
  ),
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
type ClipboardStoreState = ReturnType<typeof useClipboardStore.getState>;

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
  llm: {
    api_key: '',
    base_url: 'https://api.openai.com/v1',
    model: 'gpt-4.1-mini',
  },
};

let loadCacheStatistics: ReturnType<typeof vi.fn>;
let fetchHistory: ClipboardStoreState['fetchHistory'];
let invalidatePreview: NonNullable<ClipboardStoreState['invalidatePreview']>;
let updateConfig: ReturnType<typeof vi.fn>;
let registerGlobalShortcut: ReturnType<typeof vi.fn>;

describe('PreferencesModal actions', () => {
  beforeEach(() => {
    vi.clearAllMocks();

    loadCacheStatistics = vi.fn().mockResolvedValue(undefined);
    fetchHistory = vi.fn<ClipboardStoreState['fetchHistory']>().mockResolvedValue(undefined);
    invalidatePreview = vi.fn<NonNullable<ClipboardStoreState['invalidatePreview']>>();
    updateConfig = vi.fn().mockResolvedValue(undefined);
    registerGlobalShortcut = vi.fn().mockResolvedValue(undefined);

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
      updateConfig,
      loadCacheStatistics,
      registerGlobalShortcut,
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
    } as ClipboardStoreState);

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
          search_reindexed: 12,
          search_failed: 0,
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
      '已扫描 5 条，更新 analysis 4 条，跳过 1 条，analysis 失败 0 条；重建 search index 12 条，search 失败 0 条。'
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

  it('保存快捷键时先注册成功再写入配置', async () => {
    render(<PreferencesModal />);

    await waitFor(() => expect(loadCacheStatistics).toHaveBeenCalledTimes(1));

    fireEvent.click(screen.getByTestId('shortcut-recorder'));
    fireEvent.click(screen.getByText('common:save'));

    await waitFor(() => {
      expect(registerGlobalShortcut).toHaveBeenCalledWith('CmdOrCtrl+Alt+C');
      expect(updateConfig).toHaveBeenCalledWith(
        expect.objectContaining({
          global_shortcut: 'CmdOrCtrl+Alt+C',
        })
      );
    });

    expect(registerGlobalShortcut.mock.invocationCallOrder[0]).toBeLessThan(
      updateConfig.mock.invocationCallOrder[0]
    );
  });

  it('AI 配置测试会使用当前表单值调用测试命令并显示成功状态', async () => {
    mockedInvoke.mockImplementation(async (command, args) => {
      if (command === 'get_installed_applications') {
        return [];
      }

      if (command === 'test_llm_config') {
        expect(args).toEqual({
          config: {
            api_key: 'sk-test',
            base_url: 'https://api.openai.com/v1',
            model: 'gpt-4.1',
          },
        });

        return {
          content: 'OK',
          model: 'gpt-4.1',
        };
      }

      throw new Error(`Unexpected invoke: ${command}`);
    });

    render(<PreferencesModal />);

    await waitFor(() => expect(loadCacheStatistics).toHaveBeenCalledTimes(1));

    fireEvent.change(screen.getByPlaceholderText('ai.apiKeyPlaceholder'), {
      target: { value: 'sk-test' },
    });
    fireEvent.change(screen.getByPlaceholderText('ai.modelPlaceholder'), {
      target: { value: 'gpt-4.1' },
    });
    fireEvent.click(screen.getByTestId('llm-test-button'));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith('test_llm_config', {
        config: {
          api_key: 'sk-test',
          base_url: 'https://api.openai.com/v1',
          model: 'gpt-4.1',
        },
      });
    });

    expect(screen.getByTestId('llm-test-success')).toHaveTextContent('ai.testSuccess');
    expect(screen.getByTestId('llm-test-model')).toHaveTextContent('gpt-4.1');
    expect(updateConfig).not.toHaveBeenCalled();
  });

  it('AI 配置测试失败时显示错误信息', async () => {
    mockedInvoke.mockImplementation(async (command) => {
      if (command === 'get_installed_applications') {
        return [];
      }

      if (command === 'test_llm_config') {
        throw new Error('invalid api key');
      }

      throw new Error(`Unexpected invoke: ${command}`);
    });

    render(<PreferencesModal />);

    await waitFor(() => expect(loadCacheStatistics).toHaveBeenCalledTimes(1));

    fireEvent.change(screen.getByPlaceholderText('ai.apiKeyPlaceholder'), {
      target: { value: 'sk-test' },
    });
    fireEvent.click(screen.getByTestId('llm-test-button'));

    await waitFor(() => {
      expect(screen.getByTestId('llm-test-error')).toHaveTextContent('invalid api key');
    });
  });
});
