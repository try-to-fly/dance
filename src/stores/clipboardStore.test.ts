import { waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { ClipboardEntry } from '../types/clipboard';

const invokeMock = vi.fn();
const writeTextMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({
  writeText: (...args: unknown[]) => writeTextMock(...args),
}));

import { useClipboardStore } from './clipboardStore';

const baseEntry: ClipboardEntry = {
  id: 'entry-1',
  content_hash: 'hash-1',
  content_type: 'text/plain',
  content_data: 'hello',
  source_app: 'Terminal',
  created_at: Date.now(),
  copy_count: 1,
  file_path: null,
  is_favorite: false,
  content_subtype: 'plain_text',
  metadata: null,
  app_bundle_id: null,
};

describe('clipboardStore preview resolution', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    writeTextMock.mockReset();
    useClipboardStore.setState({
      entries: [],
      statistics: null,
      isMonitoring: false,
      searchTerm: '',
      loading: false,
      error: null,
      selectedType: 'all',
      selectedSourceApp: 'all',
      favoritesOnly: false,
      sourceAppOptions: [],
      selectedEntry: null,
      urlContentCache: new Map(),
      mediaMetadataCache: new Map(),
      previewResolutionCache: new Map(),
      hasMore: true,
      isLoadingMore: false,
    });
  });

  it('URL 解析失败时仍走前端 fallback，并使用短 TTL 缓存降级结果', async () => {
    invokeMock.mockImplementation((command: string) => {
      if (command === 'resolve_url_preview') {
        throw 'HTTP error: 503';
      }
      if (command === 'extract_media_metadata') {
        return Promise.resolve({ width: 1920, height: 1080 });
      }
      throw new Error(`unexpected command: ${command}`);
    });

    const result = await useClipboardStore
      .getState()
      .resolveUrlPreview?.('https://example.com/preview.png');

    expect(result).toMatchObject({
      imageUrl: 'https://example.com/preview.png',
      media: { width: 1920, height: 1080 },
      url: {
        finalUrl: 'https://example.com/preview.png',
        previewKind: 'image',
        error: 'HTTP error: 503',
      },
    });

    const cacheEntry = useClipboardStore
      .getState()
      .previewResolutionCache?.get('url:https://example.com/preview.png');
    expect(cacheEntry?.ttlMs).toBe(30_000);
  });

  it('普通网页解析结果优先展示后端生成的截图预览', async () => {
    invokeMock.mockImplementation((command: string) => {
      if (command === 'resolve_url_preview') {
        return Promise.resolve({
          final_url: 'https://example.com/docs',
          status: 200,
          content_type: 'text/html',
          preview_kind: 'url_card',
          title: 'Dance Documentation',
          description: 'Clipboard preview documentation for developers.',
          resolved: {
            source_kind: 'remote',
            image_url: 'data:image/png;base64,preview-screenshot',
            file_name: 'docs',
          },
        });
      }

      throw new Error(`unexpected command: ${command}`);
    });

    const result = await useClipboardStore
      .getState()
      .resolveUrlPreview?.('https://example.com/docs');

    expect(result).toMatchObject({
      imageUrl: 'data:image/png;base64,preview-screenshot',
      url: {
        finalUrl: 'https://example.com/docs',
        previewKind: 'url_card',
        title: 'Dance Documentation',
        description: 'Clipboard preview documentation for developers.',
        contentType: 'text/html',
      },
    });
    expect(result?.textContent).toBeUndefined();
  });

  it('setSelectedType 会改走后端 retrieval query，而不是继续本地同步过滤', async () => {
    invokeMock.mockImplementation((command: string, payload?: Record<string, unknown>) => {
      if (command === 'search_clipboard_history') {
        expect(payload).toEqual({
          query: {
            selected_type: 'image',
            limit: 50,
            offset: 0,
          },
        });
        return Promise.resolve([]);
      }
      if (command === 'list_clipboard_source_apps') {
        return Promise.resolve([]);
      }
      throw new Error(`unexpected command: ${command}`);
    });

    useClipboardStore.setState({
      entries: [{ ...baseEntry, content_subtype: 'plain_text' }],
      selectedEntry: baseEntry,
    });

    useClipboardStore.getState().setSelectedType('image');

    await waitFor(() => {
      expect(useClipboardStore.getState().selectedType).toBe('image');
      expect(useClipboardStore.getState().selectedEntry).toBeNull();
    });
  });

  it('copyToClipboard 统一走 backend copy_to_clipboard 合同', async () => {
    invokeMock.mockResolvedValue(undefined);

    await useClipboardStore.getState().copyToClipboard('copied from store');

    expect(invokeMock).toHaveBeenCalledWith('copy_to_clipboard', {
      content: 'copied from store',
    });
    expect(writeTextMock).not.toHaveBeenCalled();
  });

  it('URL 条目会走 resolveUrlPreview，并使用远端解析结果作为详情预览', async () => {
    const entry: ClipboardEntry = {
      ...baseEntry,
      content_data: 'https://example.com/api/data',
      content_subtype: 'url',
      analysis: {
        contract_version: 1,
        analysis_version: 1,
        status: 'matched',
        subtype: 'url',
        metadata: {
          kind: 'url',
          data: {
            protocol: 'https',
            host: 'example.com',
            path: '/api/data',
            query_params: [],
          },
        },
        diagnostics: [],
        analyzed_at: Date.now(),
      },
    };

    const resolveUrlPreview = vi.fn().mockResolvedValue({
      textContent: '{\n  "ok": true\n}',
      jsonContent: { ok: true },
      url: {
        finalUrl: 'https://example.com/api/data',
        previewKind: 'json',
      },
    });

    useClipboardStore.setState({
      resolveUrlPreview,
    });

    const resolved = await useClipboardStore.getState().resolveEntryPreview?.(entry);

    expect(resolveUrlPreview).toHaveBeenCalledWith('https://example.com/api/data');
    expect(resolved).toMatchObject({
      textContent: '{\n  "ok": true\n}',
      jsonContent: { ok: true },
      url: {
        finalUrl: 'https://example.com/api/data',
        previewKind: 'json',
      },
    });
  });

  it('resolveEntryPreview 对 URL 降级结果使用短 TTL 缓存', async () => {
    const resolveUrlPreview = vi.fn().mockResolvedValue({
      imageUrl: 'https://example.com/preview.png',
      url: {
        finalUrl: 'https://example.com/preview.png',
        previewKind: 'image',
        error: 'HTTP error: 503',
      },
    });
    const entry: ClipboardEntry = {
      ...baseEntry,
      id: 'entry-url-local',
      content_hash: 'hash-url-local',
      content_data: 'https://example.com/preview.png',
      content_subtype: 'url',
      analysis: {
        contract_version: 1,
        analysis_version: 1,
        status: 'matched',
        subtype: 'url',
        metadata: {
          kind: 'url',
          data: {
            protocol: 'https',
            host: 'example.com',
            path: '/preview.png',
            query_params: [],
          },
        },
        diagnostics: [],
        analyzed_at: Date.now(),
      },
    };

    useClipboardStore.setState({
      resolveUrlPreview,
    });

    const resolved = await useClipboardStore.getState().resolveEntryPreview?.(entry);

    expect(resolveUrlPreview).toHaveBeenCalledWith('https://example.com/preview.png');
    expect(resolved).toMatchObject({
      imageUrl: 'https://example.com/preview.png',
      url: {
        finalUrl: 'https://example.com/preview.png',
        previewKind: 'image',
        error: 'HTTP error: 503',
      },
    });

    const cacheEntry = useClipboardStore
      .getState()
      .previewResolutionCache?.get('entry:entry-url-local:hash-url-local');
    expect(cacheEntry?.ttlMs).toBe(30_000);
  });

  it('URL 条目解析前不会把 raw URL 预填进 resolved textContent', async () => {
    const entry: ClipboardEntry = {
      ...baseEntry,
      content_data: 'https://example.com/api/data',
      content_subtype: 'url',
      analysis: {
        contract_version: 1,
        analysis_version: 1,
        status: 'matched',
        subtype: 'url',
        metadata: {
          kind: 'url',
          data: {
            protocol: 'https',
            host: 'example.com',
            path: '/api/data',
            query_params: [],
          },
        },
        diagnostics: [],
        analyzed_at: Date.now(),
      },
    };

    useClipboardStore.setState({
      resolveUrlPreview: vi.fn().mockResolvedValue({
        url: {
          finalUrl: 'https://example.com/api/data',
          previewKind: 'json',
        },
      }),
    });

    const resolved = await useClipboardStore.getState().resolveEntryPreview?.(entry);

    expect(resolved?.textContent).toBeUndefined();
    expect(resolved?.jsonContent).toBeUndefined();
  });
});
