import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { ClipboardEntry, Statistics } from '../types/clipboard';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';

interface ClipboardStore {
  entries: ClipboardEntry[];
  statistics: Statistics | null;
  isMonitoring: boolean;
  searchTerm: string;
  loading: boolean;
  error: string | null;
  selectedType: string;
  selectedEntry: ClipboardEntry | null;
  urlContentCache: Map<string, { content: string; timestamp: number }>;
  mediaMetadataCache: Map<string, { metadata: any; timestamp: number }>;
  hasMore: boolean;
  isLoadingMore: boolean;

  // Actions
  startMonitoring: () => Promise<void>;
  stopMonitoring: () => Promise<void>;
  fetchHistory: (limit?: number, offset?: number) => Promise<void>;
  loadMoreEntries: () => Promise<void>;
  toggleFavorite: (id: string) => Promise<void>;
  deleteEntry: (id: string) => Promise<void>;
  clearHistory: () => Promise<void>;
  fetchStatistics: () => Promise<void>;
  copyToClipboard: (content: string) => Promise<void>;
  pasteSelectedEntry: (entry: ClipboardEntry) => Promise<void>;
  getImageUrl: (filePath: string) => Promise<string>;
  openFileWithSystem: (filePath: string) => Promise<void>;
  getAppIcon: (bundleId: string) => Promise<string | null>;
  fetchUrlContent: (url: string) => Promise<string>;
  checkFFprobeAvailable: () => Promise<boolean>;
  extractMediaMetadata: (url: string) => Promise<any>;
  setSearchTerm: (term: string) => void;
  setSelectedType: (type: string) => void;
  setSelectedEntry: (entry: ClipboardEntry | null) => void;
  getFilteredEntries: () => ClipboardEntry[];
  setupEventListener: () => void;
}

export const useClipboardStore = create<ClipboardStore>((set, get) => ({
  entries: [],
  statistics: null,
  isMonitoring: false,
  searchTerm: '',
  loading: false,
  error: null,
  selectedType: 'all',
  selectedEntry: null,
  urlContentCache: new Map(),
  mediaMetadataCache: new Map(),
  hasMore: true,
  isLoadingMore: false,

  startMonitoring: async () => {
    try {
      set({ loading: true, error: null });
      await invoke('start_monitoring');
      set({ isMonitoring: true });
      get().fetchHistory();
    } catch (error) {
      set({ error: String(error) });
    } finally {
      set({ loading: false });
    }
  },

  stopMonitoring: async () => {
    try {
      await invoke('stop_monitoring');
      set({ isMonitoring: false });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  fetchHistory: async (limit = 50, offset = 0) => {
    try {
      set({ loading: true, error: null });
      const entries = await invoke<ClipboardEntry[]>('get_clipboard_history', {
        limit,
        offset,
        search: get().searchTerm || undefined,
      });
      set({
        entries,
        hasMore: entries.length === limit,
      });
    } catch (error) {
      set({ error: String(error) });
    } finally {
      set({ loading: false });
    }
  },

  loadMoreEntries: async () => {
    const state = get();
    if (state.isLoadingMore || !state.hasMore) {
      return;
    }

    try {
      set({ isLoadingMore: true, error: null });
      const newEntries = await invoke<ClipboardEntry[]>('get_clipboard_history', {
        limit: 50,
        offset: state.entries.length,
        search: state.searchTerm || undefined,
      });

      set({
        entries: [...state.entries, ...newEntries],
        hasMore: newEntries.length === 50,
        isLoadingMore: false,
      });
    } catch (error) {
      set({ error: String(error), isLoadingMore: false });
    }
  },

  toggleFavorite: async (id: string) => {
    try {
      await invoke('toggle_favorite', { id });
      // 更新本地状态
      set((state) => ({
        entries: state.entries.map((entry) =>
          entry.id === id ? { ...entry, is_favorite: !entry.is_favorite } : entry
        ),
      }));
    } catch (error) {
      set({ error: String(error) });
    }
  },

  deleteEntry: async (id: string) => {
    try {
      await invoke('delete_entry', { id });
      set((state) => {
        const remainingEntries = state.entries.filter((entry) => entry.id !== id);
        const selectedEntry =
          state.selectedEntry?.id === id ? (remainingEntries[0] ?? null) : state.selectedEntry;

        return {
          entries: remainingEntries,
          selectedEntry,
        };
      });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  clearHistory: async () => {
    try {
      await invoke('clear_history');
      set({ entries: [], selectedEntry: null });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  fetchStatistics: async () => {
    try {
      const statistics = await invoke<Statistics>('get_statistics');
      set({ statistics });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  copyToClipboard: async (content: string) => {
    try {
      await writeText(content);
    } catch (error) {
      set({ error: String(error) });
    }
  },

  pasteSelectedEntry: async (entry: ClipboardEntry) => {
    try {
      if (entry.content_type.toLowerCase().includes('image') && entry.file_path) {
        await invoke('paste_image', { filePath: entry.file_path });
      } else if (entry.content_data) {
        await invoke('paste_text', { content: entry.content_data });
      }
    } catch (error) {
      set({ error: String(error) });
    }
  },

  getImageUrl: async (filePath: string) => {
    try {
      return await invoke<string>('get_image_url', { filePath });
    } catch (error) {
      throw new Error(String(error));
    }
  },

  openFileWithSystem: async (filePath: string) => {
    try {
      await invoke('open_file_with_system', { filePath });
    } catch (error) {
      throw new Error(String(error));
    }
  },

  getAppIcon: async (bundleId: string) => {
    try {
      return await invoke<string | null>('get_app_icon', { bundleId });
    } catch (error) {
      console.error('Failed to get app icon:', error);
      return null;
    }
  },

  fetchUrlContent: async (url: string) => {
    const state = get();
    const now = Date.now();
    const cacheExpiry = 5 * 60 * 1000; // 5分钟缓存过期时间

    // 检查缓存
    const cached = state.urlContentCache.get(url);
    if (cached && now - cached.timestamp < cacheExpiry) {
      console.log(`[fetchUrlContent] 使用缓存内容: ${url}`);
      return cached.content;
    }

    try {
      console.log(`[fetchUrlContent] 请求新内容: ${url}`);
      const content = await invoke<string>('fetch_url_content', { url });

      // 更新缓存
      set((state) => {
        const newCache = new Map(state.urlContentCache);
        newCache.set(url, { content, timestamp: now });
        return { urlContentCache: newCache };
      });

      return content;
    } catch (error) {
      throw new Error(String(error));
    }
  },

  checkFFprobeAvailable: async () => {
    try {
      console.log('[checkFFprobeAvailable] 检查 FFprobe 是否可用');
      return await invoke<boolean>('check_ffprobe_available');
    } catch (error) {
      console.error('Failed to check FFprobe availability:', error);
      return false;
    }
  },

  extractMediaMetadata: async (url: string) => {
    const state = get();
    const now = Date.now();
    const cacheExpiry = 10 * 60 * 1000; // 10分钟缓存过期时间

    // 检查缓存
    const cached = state.mediaMetadataCache.get(url);
    if (cached && now - cached.timestamp < cacheExpiry) {
      console.log(`[extractMediaMetadata] 使用缓存元数据: ${url}`);
      return cached.metadata;
    }

    try {
      console.log(`[extractMediaMetadata] 提取媒体元数据: ${url}`);
      const metadata = await invoke<any>('extract_media_metadata', { url });

      // 更新缓存
      set((state) => {
        const newCache = new Map(state.mediaMetadataCache);
        newCache.set(url, { metadata, timestamp: now });
        return { mediaMetadataCache: newCache };
      });

      return metadata;
    } catch (error) {
      console.error('Failed to extract media metadata:', error);
      throw new Error(String(error));
    }
  },

  setSearchTerm: (term: string) => {
    set({ searchTerm: term, hasMore: true });
    get().fetchHistory();
  },

  setSelectedType: (type: string) => {
    set({ selectedType: type });
    const filtered = get().getFilteredEntries();
    if (filtered.length > 0) {
      set({ selectedEntry: filtered[0] });
    }
  },

  setSelectedEntry: (entry: ClipboardEntry | null) => {
    set({ selectedEntry: entry });
  },

  getFilteredEntries: () => {
    const state = get();
    let filtered = state.entries;

    if (state.selectedType !== 'all') {
      filtered = filtered.filter((entry) => {
        const type = entry.content_type.toLowerCase();

        // 处理子类型筛选
        if (state.selectedType.startsWith('text:')) {
          if (!type.includes('text') && !type.includes('string')) {
            return false;
          }

          const subtype = state.selectedType.replace('text:', '');
          if (subtype === 'all') {
            return true;
          }

          // 检查content_subtype字段
          let entrySubtype = 'plain_text';
          if (entry.content_subtype) {
            // content_subtype直接是字符串，不需要JSON解析
            entrySubtype = entry.content_subtype;
            console.log(
              '[Filter] Entry:',
              entry.content_data?.substring(0, 20),
              'subtype:',
              entrySubtype,
              'filtering for:',
              subtype
            );
          }

          return entrySubtype === subtype;
        }

        // 处理主类型筛选
        if (state.selectedType === 'text') {
          return type.includes('text') || type.includes('string');
        } else if (state.selectedType === 'image') {
          return type.includes('image');
        } else if (state.selectedType === 'file') {
          return type.includes('file') && !type.includes('image');
        }
        return true;
      });
    }

    if (state.searchTerm) {
      const searchLower = state.searchTerm.toLowerCase();
      filtered = filtered.filter(
        (entry) =>
          entry.content_data?.toLowerCase().includes(searchLower) ||
          entry.source_app?.toLowerCase().includes(searchLower)
      );
    }

    return filtered;
  },

  setupEventListener: () => {
    listen<ClipboardEntry>('clipboard-update', (event) => {
      set((state) => {
        // 检查是否已存在
        const existingIndex = state.entries.findIndex(
          (entry) => entry.content_hash === event.payload.content_hash
        );

        let newEntries;
        let updatedEntry;

        if (existingIndex >= 0) {
          // 更新现有条目，使用后端发送的正确数据
          newEntries = [...state.entries];
          newEntries[existingIndex] = {
            ...event.payload, // 使用后端发送的完整数据，包括正确的copy_count
          };
          // 移到最前面
          const [updated] = newEntries.splice(existingIndex, 1);
          newEntries.unshift(updated);
          updatedEntry = updated;
        } else {
          // 添加新条目到最前面
          newEntries = [event.payload, ...state.entries];
          updatedEntry = event.payload;
        }

        // 自动选中最新的素材
        return {
          entries: newEntries,
          selectedEntry: updatedEntry,
        };
      });
    });
  },
}));
