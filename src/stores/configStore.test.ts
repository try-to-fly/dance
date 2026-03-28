import { beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { useConfigStore } from './configStore';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

describe('configStore cache statistics', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useConfigStore.setState({
      cacheStats: null,
      cacheStatsLoading: false,
      cacheStatsError: null,
    });
  });

  it('loadCacheStatistics 成功时会写入统计结果', async () => {
    mockedInvoke.mockResolvedValue({
      db_size_bytes: 1024,
      images_size_bytes: 2048,
      total_entries: 10,
      text_entries: 7,
      image_entries: 3,
    });

    await useConfigStore.getState().loadCacheStatistics();

    expect(mockedInvoke).toHaveBeenCalledWith('get_cache_statistics');
    expect(useConfigStore.getState().cacheStats).toEqual({
      db_size_bytes: 1024,
      images_size_bytes: 2048,
      total_entries: 10,
      text_entries: 7,
      image_entries: 3,
    });
    expect(useConfigStore.getState().cacheStatsError).toBeNull();
    expect(useConfigStore.getState().cacheStatsLoading).toBe(false);
  });

  it('loadCacheStatistics 失败时会保留错误并清空旧数据', async () => {
    useConfigStore.setState({
      cacheStats: {
        db_size_bytes: 512,
        images_size_bytes: 256,
        total_entries: 3,
        text_entries: 2,
        image_entries: 1,
      },
    });

    mockedInvoke.mockRejectedValue(new Error('stats failed'));

    await useConfigStore.getState().loadCacheStatistics();

    expect(useConfigStore.getState().cacheStats).toBeNull();
    expect(useConfigStore.getState().cacheStatsError).toContain('stats failed');
    expect(useConfigStore.getState().cacheStatsLoading).toBe(false);
  });
});
