import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import {
  Base64PreviewResolution,
  ClipboardEntry,
  PreviewKind,
  ResolvedPreviewData,
  Statistics,
  UrlPreviewResolution,
} from '../types/clipboard';
import { getEntryAnalysisSubtype } from '../lib/preview/entryPresentation';

type UrlPreviewCategory = 'none' | 'image' | 'video' | 'audio' | 'text' | 'json';

type PreviewResolutionCacheEntry = {
  data: ResolvedPreviewData;
  updatedAt: number;
  ttlMs: number;
};

const DEFAULT_PREVIEW_CACHE_TTL_MS = 5 * 60 * 1000;
const DEGRADED_PREVIEW_CACHE_TTL_MS = 30 * 1000;

const URL_MEDIA_RULES: Array<[RegExp, UrlPreviewCategory]> = [
  [/\.(jpg|jpeg|png|gif|webp|svg|bmp|ico)(\?|$)/i, 'image'],
  [/\.(mp4|webm|ogg|avi|mov|mkv|flv)(\?|$)/i, 'video'],
  [/\.(mp3|wav|flac|aac|m4a)(\?|$)/i, 'audio'],
  [/\.(json)(\?|$)/i, 'json'],
  [
    /\.(xml|html|htm|css|js|ts|jsx|tsx|py|java|cpp|c|h|php|rb|go|rs|sql|md|txt|log|csv|yaml|yml|toml|ini|conf|sh|bat)(\?|$)/i,
    'text',
  ],
];

const normalizeUrlString = (value: string): string => {
  const trimmed = value.trim();
  if (!trimmed) {
    return '';
  }

  try {
    const parsed = new URL(trimmed);
    if (parsed.protocol === 'http:' || parsed.protocol === 'https:') {
      return parsed.toString();
    }
  } catch {
    // keep fallback below
  }

  if (!/^\S+\.\S+/.test(trimmed)) {
    return '';
  }

  try {
    const parsed = new URL(`https://${trimmed}`);
    return parsed.toString();
  } catch {
    return '';
  }
};

const guessUrlPreviewCategory = (url: string): UrlPreviewCategory => {
  const lower = url.toLowerCase();
  const matched = URL_MEDIA_RULES.find(([regex]) => regex.test(lower));
  if (matched) {
    return matched[1];
  }
  if (lower.includes('/api/')) {
    return 'json';
  }
  return 'none';
};

const inferUrlTextPreviewKind = (url: string): PreviewKind => {
  const lower = url.toLowerCase();
  if (/\.(json)(\?|$)/i.test(lower) || lower.includes('/api/')) {
    return 'json';
  }
  if (/\.(md|markdown)(\?|$)/i.test(lower)) {
    return 'markdown';
  }
  if (/\.(sh|bat)(\?|$)/i.test(lower)) {
    return 'code';
  }
  if (/\.(html|htm|css|js|jsx|ts|tsx|py|java|cpp|c|h|php|rb|go|rs|sql|xml)(\?|$)/i.test(lower)) {
    return 'code';
  }

  return 'plain_text';
};

const decodeBase64String = (input: string) => {
  const trimmed = input.trim();
  const dataUriMatch = trimmed.match(/^data:([^;,]+);base64,(.+)$/i);
  const mime = dataUriMatch?.[1];
  const rawData = (dataUriMatch?.[2] || trimmed).replace(/\s+/g, '');
  const binary = atob(rawData);
  const bytes = Uint8Array.from(binary, (char) => char.charCodeAt(0));

  return { mime, bytes, rawData };
};

const decodeBase64Fallback = (input: string): ResolvedPreviewData['base64'] | undefined => {
  try {
    const { mime: dataMime, bytes, rawData } = decodeBase64String(input);
    const header = `${bytes[0] ?? 0},${bytes[1] ?? 0},${bytes[2] ?? 0},${bytes[3] ?? 0}`;
    const decoder = new TextDecoder('utf-8', { fatal: false });
    const textCandidate = decoder.decode(bytes);

    const mime = (() => {
      if (dataMime) {
        return dataMime;
      }
      if (header === '137,80,78,71') {
        return 'image/png';
      }
      if (header === '255,216,255,224' || header === '255,216,255,225') {
        return 'image/jpeg';
      }
      if (textCandidate.trim().startsWith('{') || textCandidate.trim().startsWith('[')) {
        return 'application/json';
      }
      return 'application/octet-stream';
    })();

    if (mime.startsWith('image/')) {
      return {
        decodedKind: 'image',
        mime,
        dataUrl: `data:${mime};base64,${rawData}`,
        sizeBytes: bytes.byteLength,
      };
    }
    if (mime.startsWith('audio/')) {
      return {
        decodedKind: 'audio',
        mime,
        dataUrl: `data:${mime};base64,${rawData}`,
        sizeBytes: bytes.byteLength,
      };
    }
    if (mime.startsWith('video/')) {
      return {
        decodedKind: 'video',
        mime,
        dataUrl: `data:${mime};base64,${rawData}`,
        sizeBytes: bytes.byteLength,
      };
    }

    try {
      const json = JSON.parse(textCandidate);
      return {
        decodedKind: 'json',
        mime: 'application/json',
        jsonContent: json,
        textPreview: JSON.stringify(json, null, 2),
        sizeBytes: bytes.byteLength,
      };
    } catch {
      // keep text/binary fallback
    }

    const printableRatio =
      textCandidate.length === 0
        ? 0
        : textCandidate.replace(/[\x20-\x7E\n\r\t]/g, '').length / textCandidate.length;
    if (printableRatio < 0.1) {
      return {
        decodedKind: 'text',
        mime: 'text/plain',
        textPreview: textCandidate,
        sizeBytes: bytes.byteLength,
      };
    }

    return {
      decodedKind: 'binary',
      mime,
      sizeBytes: bytes.byteLength,
    };
  } catch {
    return undefined;
  }
};

const getPreviewCacheTtlMs = (resolvedData: ResolvedPreviewData) =>
  resolvedData.url?.error ? DEGRADED_PREVIEW_CACHE_TTL_MS : DEFAULT_PREVIEW_CACHE_TTL_MS;

const hasResolvedJsonContent = (
  resolvedData?: Pick<ResolvedPreviewData, 'jsonContent'>
): resolvedData is Pick<ResolvedPreviewData, 'jsonContent'> & { jsonContent: unknown } =>
  resolvedData?.jsonContent !== undefined;

const mapMediaPreviewInfo = (
  media?:
    | NonNullable<UrlPreviewResolution['resolved']>['media']
    | NonNullable<Base64PreviewResolution['resolved']>['media']
): ResolvedPreviewData['media'] | undefined => {
  if (!media) {
    return undefined;
  }

  return {
    duration: media.duration,
    bitrate: media.bitrate,
    codec: media.codec,
    width: media.width,
    height: media.height,
    fps: media.fps,
    sampleRate: media.sample_rate,
    size:
      typeof media.size_bytes === 'number'
        ? `${Math.round(media.size_bytes / 1024)} KB`
        : undefined,
    format: media.format,
  };
};

const mapResolvedPreviewData = (
  resolved?: UrlPreviewResolution['resolved'] | Base64PreviewResolution['resolved']
): ResolvedPreviewData => {
  if (!resolved) {
    return {};
  }

  return {
    sourceKind:
      resolved.source_kind === 'remote' || resolved.source_kind === 'decoded'
        ? resolved.source_kind
        : 'local',
    mime: resolved.mime,
    fileName: resolved.file_name,
    extension: resolved.extension,
    sizeBytes: resolved.size_bytes,
    textContent: resolved.text_content,
    jsonContent: resolved.json_content,
    imageUrl: resolved.image_url,
    audioUrl: resolved.audio_url,
    videoUrl: resolved.video_url,
    media: mapMediaPreviewInfo(resolved.media),
    base64: resolved.base64
      ? {
          decodedKind: resolved.base64.decoded_kind || 'unknown',
          mime: resolved.base64.mime,
          textPreview: resolved.base64.text_preview,
          dataUrl: resolved.base64.data_url,
        }
      : undefined,
  };
};

const hasRenderableResolvedPreview = (resolved: ResolvedPreviewData) =>
  Boolean(resolved.imageUrl || resolved.audioUrl || resolved.videoUrl) ||
  resolved.textContent !== undefined ||
  hasResolvedJsonContent(resolved);

export const copyToClipboard = async (content: string) => {
  await invoke('copy_to_clipboard', { content });
};

const applyUrlPreviewFallback = async ({
  normalizedUrl,
  resolved,
  error,
  fetchUrlContent,
  extractMediaMetadata,
}: {
  normalizedUrl: string;
  resolved: ResolvedPreviewData;
  error: string;
  fetchUrlContent: (url: string) => Promise<string>;
  extractMediaMetadata: (url: string) => Promise<any>;
}): Promise<ResolvedPreviewData> => {
  const finalUrl = resolved.url?.finalUrl || normalizedUrl;
  const category = guessUrlPreviewCategory(finalUrl);
  const previewKind =
    category === 'image'
      ? 'image'
      : category === 'video'
        ? 'video'
        : category === 'audio'
          ? 'audio'
          : category === 'json'
            ? 'json'
            : category === 'text'
              ? inferUrlTextPreviewKind(finalUrl)
              : 'url_card';

  const nextResolved: ResolvedPreviewData = {
    ...resolved,
    url: {
      ...resolved.url,
      error,
      finalUrl,
      previewKind,
    },
  };

  if (category === 'image') {
    nextResolved.imageUrl = finalUrl;
  } else if (category === 'video') {
    nextResolved.videoUrl = finalUrl;
  } else if (category === 'audio') {
    nextResolved.audioUrl = finalUrl;
  } else if (category === 'text' || category === 'json') {
    try {
      const content = await fetchUrlContent(finalUrl);
      if (category === 'json') {
        try {
          const parsed = JSON.parse(content);
          nextResolved.jsonContent = parsed;
          nextResolved.textContent = JSON.stringify(parsed, null, 2);
        } catch {
          nextResolved.textContent = content;
        }
      } else {
        nextResolved.textContent = content;
      }
    } catch (fetchError) {
      console.error('[resolveUrlPreview] fallback 获取失败:', fetchError);
    }
  }

  if (category === 'image' || category === 'video' || category === 'audio') {
    try {
      const metadata = await extractMediaMetadata(finalUrl);
      nextResolved.media = {
        width: Number(metadata.width) || undefined,
        height: Number(metadata.height) || undefined,
        duration: metadata.duration,
        bitrate: metadata.bitrate,
        codec: metadata.codec,
        fps: metadata.fps ? String(metadata.fps) : undefined,
        sampleRate: metadata.sample_rate ? String(metadata.sample_rate) : undefined,
      };
    } catch (mediaError) {
      console.error('[resolveUrlPreview] 媒体探测失败:', mediaError);
    }
  }

  return nextResolved;
};

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
  previewResolutionCache?: Map<string, PreviewResolutionCacheEntry>;
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
  resolveUrlPreview?: (url: string) => Promise<ResolvedPreviewData>;
  decodeBase64Preview?: (input: string) => Promise<ResolvedPreviewData>;
  resolveEntryPreview?: (entry: ClipboardEntry) => Promise<ResolvedPreviewData>;
  invalidatePreview?: (key?: string) => void;
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
  previewResolutionCache: new Map(),
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
      set((state) => {
        const selectedEntry =
          state.selectedEntry && entries.some((entry) => entry.id === state.selectedEntry?.id)
            ? state.selectedEntry
            : (entries[0] ?? null);

        return {
          entries,
          hasMore: entries.length === limit,
          selectedEntry,
        };
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
      await copyToClipboard(content);
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

  resolveUrlPreview: async (url: string) => {
    const normalizedUrl = normalizeUrlString(url);
    if (!normalizedUrl) {
      return {};
    }

    const state = get();
    const cacheKey = `url:${normalizedUrl}`;
    const now = Date.now();
    const cache = state.previewResolutionCache;
    const cached = cache?.get(cacheKey);
    if (cached && now - cached.updatedAt < (cached.ttlMs ?? DEFAULT_PREVIEW_CACHE_TTL_MS)) {
      return cached.data;
    }

    let resolved: ResolvedPreviewData = {
      sourceKind: 'remote',
      url: {
        finalUrl: normalizedUrl,
      },
    };

    try {
      const response = await invoke<UrlPreviewResolution>('resolve_url_preview', {
        url: normalizedUrl,
      });
      const resolvedFromBackend = mapResolvedPreviewData(response.resolved);
      resolved = {
        ...resolved,
        ...resolvedFromBackend,
        mime: resolvedFromBackend.mime ?? response.content_type,
        url: {
          finalUrl: response.final_url || normalizedUrl,
          status: response.status,
          contentType: response.content_type,
          contentLength: response.content_length,
          title: resolvedFromBackend.fileName,
          previewKind: response.preview_kind,
          error: response.error,
        },
      };

      if (response.error && !hasRenderableResolvedPreview(resolved)) {
        resolved = await applyUrlPreviewFallback({
          normalizedUrl,
          resolved,
          error: response.error,
          fetchUrlContent: get().fetchUrlContent,
          extractMediaMetadata: get().extractMediaMetadata,
        });
        console.warn('[resolveUrlPreview] 使用前端 fallback:', response.error);
      }
    } catch (error) {
      const errorMessage = String(error);
      resolved.url = {
        ...resolved.url,
        finalUrl: normalizedUrl,
        error: errorMessage,
      };
      resolved = await applyUrlPreviewFallback({
        normalizedUrl,
        resolved,
        error: errorMessage,
        fetchUrlContent: get().fetchUrlContent,
        extractMediaMetadata: get().extractMediaMetadata,
      });
      console.warn('[resolveUrlPreview] 使用前端 fallback:', error);
    }

    const ttlMs = getPreviewCacheTtlMs(resolved);
    set((current) => {
      const next = new Map(current.previewResolutionCache ?? new Map());
      next.set(cacheKey, { data: resolved, updatedAt: now, ttlMs });
      return { previewResolutionCache: next };
    });

    return resolved;
  },

  decodeBase64Preview: async (input: string) => {
    const normalized = input.trim();
    if (!normalized) {
      return {};
    }

    try {
      const response = await invoke<Base64PreviewResolution>('decode_base64_preview', {
        input: normalized,
      });
      const resolved = mapResolvedPreviewData(response.resolved);
      const inlineMediaUrl =
        resolved.base64?.dataUrl || resolved.imageUrl || resolved.audioUrl || resolved.videoUrl;
      const degradedMedia =
        Boolean(response.error) &&
        !inlineMediaUrl &&
        (response.decoded_kind === 'image' ||
          response.decoded_kind === 'audio' ||
          response.decoded_kind === 'video');

      return {
        ...resolved,
        base64: {
          decodedKind: degradedMedia
            ? 'binary'
            : response.decoded_kind || resolved.base64?.decodedKind || 'unknown',
          mime: resolved.base64?.mime ?? resolved.mime,
          textPreview: resolved.base64?.textPreview ?? resolved.textContent,
          jsonContent: resolved.jsonContent,
          dataUrl: degradedMedia ? undefined : inlineMediaUrl,
          filenameSuggestion: response.filename_suggestion,
          sizeBytes: resolved.sizeBytes,
          error: response.error,
        },
        imageUrl: degradedMedia ? undefined : resolved.imageUrl,
        audioUrl: degradedMedia ? undefined : resolved.audioUrl,
        videoUrl: degradedMedia ? undefined : resolved.videoUrl,
      };
    } catch (error) {
      console.warn('[decodeBase64Preview] 使用前端 fallback:', error);
      const fallback = decodeBase64Fallback(normalized);
      return fallback
        ? {
            sourceKind: 'decoded',
            mime: fallback.mime,
            sizeBytes: fallback.sizeBytes,
            textContent: fallback.textPreview,
            jsonContent: fallback.jsonContent,
            imageUrl: fallback.decodedKind === 'image' ? fallback.dataUrl : undefined,
            audioUrl: fallback.decodedKind === 'audio' ? fallback.dataUrl : undefined,
            videoUrl: fallback.decodedKind === 'video' ? fallback.dataUrl : undefined,
            base64: fallback,
          }
        : {};
    }
  },

  resolveEntryPreview: async (entry: ClipboardEntry) => {
    const state = get();
    const cacheKey = `entry:${entry.id}:${entry.content_hash}`;
    const now = Date.now();
    const cached = state.previewResolutionCache?.get(cacheKey);
    if (cached && now - cached.updatedAt < (cached.ttlMs ?? DEFAULT_PREVIEW_CACHE_TTL_MS)) {
      return cached.data;
    }

    const resolved: ResolvedPreviewData = {};
    const contentType = entry.content_type.toLowerCase();
    const subType = getEntryAnalysisSubtype(entry);
    let ttlMs = DEFAULT_PREVIEW_CACHE_TTL_MS;

    if (contentType.includes('image') && entry.file_path) {
      try {
        resolved.sourceKind = 'local';
        resolved.imageUrl = await state.getImageUrl(entry.file_path);
      } catch (error) {
        console.error('[resolveEntryPreview] 图片加载失败:', error);
      }
    }

    if ((contentType.includes('text') || contentType.includes('string')) && entry.content_data) {
      if (subType === 'url' && state.resolveUrlPreview) {
        const urlResolved = await state.resolveUrlPreview(entry.content_data);
        Object.assign(resolved, urlResolved);
        ttlMs = Math.min(ttlMs, getPreviewCacheTtlMs(urlResolved));
      } else {
        resolved.textContent = entry.content_data;
        if (subType === 'json') {
          try {
            resolved.jsonContent = JSON.parse(entry.content_data);
          } catch {
            // keep raw text only
          }
        }
      }
      if (subType === 'base64' && state.decodeBase64Preview) {
        const base64Resolved = await state.decodeBase64Preview(entry.content_data);
        Object.assign(resolved, base64Resolved);
      }
    }

    set((current) => {
      const next = new Map(current.previewResolutionCache ?? new Map());
      next.set(cacheKey, { data: resolved, updatedAt: now, ttlMs });
      return { previewResolutionCache: next };
    });

    return resolved;
  },

  invalidatePreview: (key?: string) => {
    if (!key) {
      set({ previewResolutionCache: new Map() });
      return;
    }

    set((state) => {
      const next = new Map(state.previewResolutionCache ?? new Map());
      next.delete(key);
      return { previewResolutionCache: next };
    });
  },

  setSearchTerm: (term: string) => {
    set({ searchTerm: term, hasMore: true });
    get().fetchHistory();
  },

  setSelectedType: (type: string) => {
    set({ selectedType: type });
    const filtered = get().getFilteredEntries();
    set({ selectedEntry: filtered[0] ?? null });
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
          const entrySubtype = getEntryAnalysisSubtype(entry);

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
