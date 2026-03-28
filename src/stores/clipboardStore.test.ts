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

  it('setSelectedType 在过滤结果为空时会清空 selectedEntry', () => {
    useClipboardStore.setState({
      entries: [{ ...baseEntry, content_subtype: 'plain_text' }],
      selectedEntry: baseEntry,
    });

    useClipboardStore.getState().setSelectedType('image');

    expect(useClipboardStore.getState().selectedEntry).toBeNull();
  });

  it('copyToClipboard 统一走 backend copy_to_clipboard 合同', async () => {
    invokeMock.mockResolvedValue(undefined);

    await useClipboardStore.getState().copyToClipboard('copied from store');

    expect(invokeMock).toHaveBeenCalledWith('copy_to_clipboard', {
      content: 'copied from store',
    });
    expect(writeTextMock).not.toHaveBeenCalled();
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

    expect(resolved).toMatchObject({
      url: {
        finalUrl: 'https://example.com/api/data',
        previewKind: 'json',
      },
    });
    expect(resolved?.textContent).toBeUndefined();
    expect(resolved?.jsonContent).toBeUndefined();
  });
});
