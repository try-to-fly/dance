import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useClipboardStore } from '../../stores/clipboardStore';
import { ClipboardEntry, EntryAnalysisSnapshot, ResolvedPreviewData } from '../../types/clipboard';
import { DetailView } from './DetailView';

const mockedComponents = vi.hoisted(() => ({
  unifiedTextRenderer: vi.fn(
    ({
      content,
      contentSubType,
      onContentChange,
      sessionKey,
    }: {
      content: string;
      contentSubType: string;
      onContentChange?: (value: string) => void;
      sessionKey?: string;
    }) => (
      <div data-testid="renderer-unified">
        {contentSubType}:{content}
        {sessionKey ? <span data-testid="renderer-session-key">{sessionKey}</span> : null}
        {onContentChange ? (
          <button onClick={() => onContentChange('editedValue')} type="button">
            mutate-workbench
          </button>
        ) : null}
      </div>
    )
  ),
  urlRenderer: vi.fn(({ content }: { content: string }) => (
    <div data-testid="renderer-url">{content}</div>
  )),
  colorRenderer: vi.fn(({ content }: { content: string }) => (
    <div data-testid="renderer-color">{content}</div>
  )),
  ipRenderer: vi.fn(({ content }: { content: string }) => (
    <div data-testid="renderer-ip">{content}</div>
  )),
  emailRenderer: vi.fn(({ content }: { content: string }) => (
    <div data-testid="renderer-email">{content}</div>
  )),
  timeRenderer: vi.fn(({ content }: { content: string }) => (
    <div data-testid="renderer-time">{content}</div>
  )),
  jsonRenderer: vi.fn(({ content }: { content: string }) => (
    <div data-testid="renderer-json">{content}</div>
  )),
  imagePreview: vi.fn(
    ({
      imageUrl,
      filePath,
      metadata,
    }: {
      imageUrl: string;
      filePath: string;
      metadata?: { width: number; height: number; file_size: number };
    }) => (
      <div data-meta={JSON.stringify(metadata)} data-testid="image-preview">
        {imageUrl}|{filePath}
      </div>
    )
  ),
  openAiDialog: vi.fn(),
  openAiChatWindow: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('../../stores/clipboardStore', () => ({
  useClipboardStore: vi.fn(),
}));

vi.mock('../../stores/aiStore', () => ({
  useAiStore: {
    getState: vi.fn(() => ({
      openDialog: mockedComponents.openAiDialog,
    })),
  },
}));

vi.mock('../../lib/ai/chatWindow', () => ({
  openAiChatWindow: mockedComponents.openAiChatWindow,
}));

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const dictionary: Record<string, string> = {
        'detail.title': '详情预览',
        'detail.selectItem': '请选择一条内容',
        'detail.type': '类型',
        'detail.source': '来源',
        'detail.time': '时间',
        'detail.copyCount': '复制次数',
        'detail.loading': '加载中',
        'detail.unknown': '未知',
        'detail.actions.copyDecoded': '复制解码内容',
        'detail.actions.openFile': '打开文件',
        'detail.ai.translate': '翻译成中文',
        'detail.ai.chat': '对话',
        'detail.analysisStatus': '分析',
        'detail.contentTypes.plain_text': '文本',
        'detail.contentTypes.command': '命令',
        'detail.contentTypes.json': 'JSON',
        'detail.contentTypes.url': 'URL',
        'detail.contentTypes.base64': 'Base64编码',
        'detail.contentTypes.image': '图片',
        'detail.contentTypes.file': '文件',
        'renderers.url.open': '打开链接',
        'clipboard:actions.favorite': '收藏',
        'clipboard:actions.unfavorite': '取消收藏',
        copy: '复制',
        paste: '粘贴',
        delete: '删除',
      };

      return dictionary[key] ?? key;
    },
  }),
}));

vi.mock('../AI/AIAssistantDialog', () => ({
  AIAssistantDialog: () => <div data-testid="ai-assistant-dialog" />,
}));

vi.mock('./ContentRenderers', () => ({
  UnifiedTextRenderer: mockedComponents.unifiedTextRenderer,
  UrlRenderer: mockedComponents.urlRenderer,
  ColorRenderer: mockedComponents.colorRenderer,
  IpRenderer: mockedComponents.ipRenderer,
  EmailRenderer: mockedComponents.emailRenderer,
  TimeRenderer: mockedComponents.timeRenderer,
  JsonRenderer: mockedComponents.jsonRenderer,
}));

vi.mock('./ImagePreview', () => ({
  ImagePreview: mockedComponents.imagePreview,
}));

const mockedUseClipboardStore = vi.mocked(useClipboardStore);

const baseEntry: ClipboardEntry = {
  id: 'entry-1',
  content_hash: 'hash-1',
  content_type: 'text/plain',
  content_data: 'echo hello',
  source_app: 'Terminal',
  created_at: new Date('2026-03-26T10:20:30Z').getTime(),
  copy_count: 7,
  file_path: null,
  is_favorite: false,
  content_subtype: 'command',
  metadata: null,
  app_bundle_id: null,
};

const createAnalysis = (overrides: Partial<EntryAnalysisSnapshot>): EntryAnalysisSnapshot => ({
  contract_version: 1,
  analysis_version: 1,
  status: 'matched',
  subtype: 'plain_text',
  metadata: {
    kind: 'plain_text',
    data: {
      char_count: 10,
      line_count: 1,
    },
  },
  diagnostics: [],
  analyzed_at: new Date('2026-03-26T10:20:30Z').getTime(),
  ...overrides,
});

const createStoreState = (selectedEntry: ClipboardEntry | null) => ({
  selectedEntry,
  getImageUrl: vi.fn().mockResolvedValue('tauri://image-preview'),
  openFileWithSystem: vi.fn(),
  copyToClipboard: vi.fn(),
  pasteSelectedEntry: vi.fn(),
  toggleFavorite: vi.fn(),
  deleteEntry: vi.fn(),
  resolveEntryPreview: undefined,
});

const formatTimestamp = (value: number, compact = false) =>
  new Date(value).toLocaleString(undefined, {
    year: compact ? undefined : 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: compact ? undefined : '2-digit',
  });

describe('DetailView', () => {
  // PREV-04 read-only wording is overridden by D-14..D-17 for code/command detail workbench behavior.
  beforeEach(() => {
    mockedUseClipboardStore.mockReturnValue(createStoreState(null));
    mockedComponents.openAiDialog.mockReset();
    mockedComponents.openAiChatWindow.mockClear();
  });

  it('在未选中内容时展示空状态', () => {
    render(<DetailView />);

    expect(screen.getByText('请选择一条内容')).toBeInTheDocument();
  });

  it('将类型放到紧凑标签中，并统一元信息 pills 为仅图标和值', () => {
    mockedUseClipboardStore.mockReturnValue(
      createStoreState({
        ...baseEntry,
        content_data: 'npm run dev',
        copy_count: 12,
      })
    );

    render(<DetailView />);

    expect(screen.getAllByTestId('renderer-unified')[0]).toHaveTextContent('command:npm run dev');
    expect(document.getElementById('detail-view-type-badge')).toHaveTextContent('命令');
    expect(document.getElementById('detail-view-title')).not.toBeInTheDocument();
    expect(document.getElementById('detail-view-metadata')).toHaveClass('flex', 'flex-wrap');
    expect(document.getElementById('detail-view-metadata')?.children).toHaveLength(3);
    expect(screen.getAllByText('Terminal').length).toBeGreaterThanOrEqual(1);
    expect(
      screen.getAllByText(formatTimestamp(baseEntry.created_at, true)).length
    ).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText('12').length).toBeGreaterThanOrEqual(1);
    expect(screen.queryByText('类型')).not.toBeInTheDocument();
    expect(screen.queryByText('来源')).not.toBeInTheDocument();
    expect(screen.queryByText('时间')).not.toBeInTheDocument();
    expect(screen.queryByText('复制次数')).not.toBeInTheDocument();
    expect(document.getElementById('detail-view-actions')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '翻译成中文' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '对话' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '更多操作' })).toBeInTheDocument();
  });

  it('翻译和对话按钮会带着当前原始文本打开工作台', async () => {
    mockedUseClipboardStore.mockReturnValue(
      createStoreState({
        ...baseEntry,
        content_data: 'npm run dev',
      })
    );

    render(<DetailView />);

    fireEvent.click(screen.getByRole('button', { name: '翻译成中文' }));
    fireEvent.click(screen.getByRole('button', { name: '对话' }));

    await waitFor(() => {
      expect(mockedComponents.openAiDialog).toHaveBeenNthCalledWith(1, {
        sourceKey: 'entry-1:hash-1',
        title: 'npm run dev',
        sourceText: 'npm run dev',
        mode: 'translate',
      });
      expect(mockedComponents.openAiChatWindow).toHaveBeenNthCalledWith(
        1,
        {
          sourceKey: 'entry-1:hash-1',
          title: 'npm run dev',
          sourceText: 'npm run dev',
        },
        {
          windowTitle: '基于原文继续对话',
        }
      );
    });
  });

  it('non-immersive JSON detail 保留 Raw tab，并使用顺序滚动布局', () => {
    mockedUseClipboardStore.mockReturnValue(
      createStoreState({
        ...baseEntry,
        content_subtype: 'json',
        content_data: '{"hello":"world"}',
      })
    );

    render(<DetailView />);

    expect(screen.getByTestId('renderer-json')).toHaveTextContent('{"hello":"world"}');
    expect(screen.getByRole('tab', { name: 'Raw' })).toBeInTheDocument();
    expect(
      screen
        .getAllByTestId('renderer-unified')
        .some((node) => node.textContent?.includes('plain_text:{"hello":"world"}'))
    ).toBe(true);
    expect(document.getElementById('detail-view-content-wrapper')).toHaveClass('flex-col');
    expect(document.getElementById('detail-view-content-wrapper')).toHaveClass('overflow-y-auto');
    expect(document.getElementById('detail-view-primary-column')).toHaveClass('shrink-0');
  });

  it('Base64 条目在新预览模型落地前保留可读的文本兜底', () => {
    mockedUseClipboardStore.mockReturnValue(
      createStoreState({
        ...baseEntry,
        content_subtype: 'base64',
        content_data: 'eyJ0eXBlIjoianNvbiJ9',
      })
    );

    render(<DetailView />);

    expect(document.getElementById('detail-view-type-badge')).toHaveTextContent('Base64编码');
    expect(
      screen
        .getAllByTestId('renderer-unified')
        .some((node) => node.textContent?.includes('plain_text:eyJ0eXBlIjoianNvbiJ9'))
    ).toBe(true);
  });

  it('fallback analysis 会保留 raw 文本主视图并显示降级提示', () => {
    mockedUseClipboardStore.mockReturnValue(
      createStoreState({
        ...baseEntry,
        content_subtype: 'json',
        content_data: '{broken-json',
        metadata: JSON.stringify({
          timestamp_formats: {
            unix_ms: 1735699200000,
          },
        }),
        analysis: createAnalysis({
          status: 'fallback',
          subtype: 'plain_text',
          diagnostics: [
            {
              code: 'json_malformed',
              severity: 'error',
              message: 'json parse failed',
            },
          ],
        }),
      })
    );

    render(<DetailView />);

    expect(screen.getAllByTestId('renderer-unified')[0]).toHaveTextContent(
      'plain_text:{broken-json'
    );
    expect(screen.getAllByText('Fallback').length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText('Detection')).toBeInTheDocument();
    expect(screen.getAllByText('Shown as plain text').length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText('Looks like JSON, but the format is incomplete')).toBeInTheDocument();
  });

  it('为图片条目异步加载预览内容和元数据', async () => {
    const store = createStoreState({
      ...baseEntry,
      content_type: 'image/png',
      content_data: null,
      file_path: '/tmp/preview.png',
      content_subtype: 'plain_text',
      metadata: JSON.stringify({
        image_metadata: {
          width: 1440,
          height: 900,
          file_size: 4096,
        },
      }),
    });

    mockedUseClipboardStore.mockReturnValue(store);

    render(<DetailView />);

    await waitFor(() => {
      expect(screen.getByTestId('image-preview')).toHaveTextContent(
        'tauri://image-preview|/tmp/preview.png'
      );
    });

    expect(store.getImageUrl).toHaveBeenCalledWith('/tmp/preview.png');
    expect(screen.getByTestId('image-preview')).toHaveAttribute(
      'data-meta',
      JSON.stringify({
        width: 1440,
        height: 900,
        file_size: 4096,
      })
    );
  });

  it('图片条目会把 base64 图片一起带入 AI 对话', async () => {
    const store = {
      ...createStoreState({
        ...baseEntry,
        content_type: 'image/png',
        content_data: null,
        file_path: '/tmp/preview.png',
      }),
      getImageUrl: vi.fn().mockResolvedValue('data:image/png;base64,preview-image'),
    };

    mockedUseClipboardStore.mockReturnValue(store);

    render(<DetailView />);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: '对话' })).toBeInTheDocument();
    });

    expect(screen.queryByRole('button', { name: '翻译成中文' })).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: '对话' }));

    await waitFor(() => {
      expect(mockedComponents.openAiChatWindow).toHaveBeenCalledWith(
        expect.objectContaining({
          sourceKey: 'entry-1:hash-1',
          sourceText: '',
          sourceImageDataUrl: 'data:image/png;base64,preview-image',
        }),
        {
          windowTitle: '基于原文继续对话',
        }
      );
    });
  });

  it('切换条目时不会短暂复用上一条的解析结果', async () => {
    const firstEntry: ClipboardEntry = {
      ...baseEntry,
      id: 'entry-url',
      content_hash: 'hash-url',
      content_subtype: 'url',
      content_data: 'https://example.com/a.png',
    };
    const secondEntry: ClipboardEntry = {
      ...baseEntry,
      id: 'entry-command',
      content_hash: 'hash-command',
      content_subtype: 'command',
      content_data: 'echo second',
    };

    let currentEntry = firstEntry;
    let resolveSecondEntry: ((value: ResolvedPreviewData) => void) | undefined;
    const resolveEntryPreview = vi.fn(async (entry: ClipboardEntry) => {
      if (entry.id === firstEntry.id) {
        return {
          imageUrl: 'https://example.com/a.png',
          url: { previewKind: 'image' },
        };
      }

      return new Promise<ResolvedPreviewData>((resolve) => {
        resolveSecondEntry = resolve;
      });
    });

    mockedUseClipboardStore.mockImplementation(
      () =>
        ({
          ...createStoreState(currentEntry),
          resolveEntryPreview,
        }) as ReturnType<typeof createStoreState> & {
          resolveEntryPreview: typeof resolveEntryPreview;
        }
    );

    const { rerender } = render(<DetailView />);

    await waitFor(() => {
      expect(screen.getByAltText('preview')).toHaveAttribute('src', 'https://example.com/a.png');
    });

    currentEntry = secondEntry;
    rerender(<DetailView />);

    expect(screen.queryByAltText('preview')).not.toBeInTheDocument();
    expect(screen.getAllByTestId('renderer-unified')[0]).toHaveTextContent('command:echo second');

    resolveSecondEntry?.({});
  });

  it('URL 指向 JSON 内容时，详情页直接展示 JSON 预览', async () => {
    const resolveEntryPreview = vi.fn().mockResolvedValue({
      textContent: '{\n  "ok": true\n}',
      jsonContent: { ok: true },
      url: {
        finalUrl: 'https://example.com/data.json',
        previewKind: 'json',
      },
    });

    mockedUseClipboardStore.mockReturnValue({
      ...createStoreState({
        ...baseEntry,
        id: 'entry-json-url',
        content_hash: 'hash-json-url',
        content_subtype: 'url',
        content_data: 'https://example.com/data.json',
      }),
      resolveEntryPreview,
    } as ReturnType<typeof createStoreState> & {
      resolveEntryPreview: typeof resolveEntryPreview;
    });

    render(<DetailView />);

    await waitFor(() => {
      expect(screen.getByTestId('renderer-json')).toHaveTextContent('"ok": true');
    });

    expect(resolveEntryPreview).toHaveBeenCalledWith(
      expect.objectContaining({
        id: 'entry-json-url',
        content_data: 'https://example.com/data.json',
      })
    );
  });

  it('URL 图片详情会展示媒体属性信息', async () => {
    const resolveEntryPreview = vi.fn().mockResolvedValue({
      imageUrl: 'https://example.com/preview.png',
      sizeBytes: 2 * 1024 * 1024,
      mime: 'image/png',
      extension: 'png',
      media: {
        width: 1920,
        height: 1080,
        format: 'png',
      },
      url: {
        finalUrl: 'https://example.com/preview.png',
        previewKind: 'image',
        contentLength: 2 * 1024 * 1024,
      },
    });

    mockedUseClipboardStore.mockReturnValue({
      ...createStoreState({
        ...baseEntry,
        id: 'entry-image-url',
        content_hash: 'hash-image-url',
        content_subtype: 'url',
        content_data: 'https://example.com/preview.png',
      }),
      resolveEntryPreview,
    } as ReturnType<typeof createStoreState> & {
      resolveEntryPreview: typeof resolveEntryPreview;
    });

    render(<DetailView />);

    await waitFor(() => {
      expect(screen.getByAltText('preview')).toHaveAttribute(
        'src',
        'https://example.com/preview.png'
      );
    });

    expect(screen.getByText('Media')).toBeInTheDocument();
    expect(screen.getByText('Resolution')).toBeInTheDocument();
    expect(screen.getByText('1920x1080')).toBeInTheDocument();
    expect(screen.getByText('Size')).toBeInTheDocument();
    expect(screen.getByText('2.0 MB')).toBeInTheDocument();
    expect(screen.getByText('MIME')).toBeInTheDocument();
    expect(screen.getByText('image/png')).toBeInTheDocument();
  });

  it('D-15 / D-16 / D-17: code detail 复制当前 workbench buffer，关闭后会重置本地编辑', async () => {
    const firstEntry: ClipboardEntry = {
      ...baseEntry,
      id: 'entry-code-a',
      content_hash: 'hash-code-a',
      content_subtype: 'code',
      content_data: 'const answer = 42;',
    };
    const reopenedEntry: ClipboardEntry = {
      ...baseEntry,
      id: 'entry-code-b',
      content_hash: 'hash-code-b',
      content_subtype: 'code',
      content_data: 'const answer = 42;',
    };
    const copyToClipboard = vi.fn().mockResolvedValue(undefined);
    let currentEntry: ClipboardEntry | null = firstEntry;
    const sharedStore = {
      ...createStoreState(firstEntry),
      copyToClipboard,
    } as ReturnType<typeof createStoreState>;

    mockedUseClipboardStore.mockImplementation(() => {
      sharedStore.selectedEntry = currentEntry;
      return sharedStore;
    });

    const { rerender } = render(<DetailView />);

    expect(screen.getByTestId('renderer-session-key')).toHaveTextContent(
      'entry-code-a:hash-code-a'
    );

    fireEvent.click(screen.getAllByRole('button', { name: 'mutate-workbench' })[0]);
    fireEvent.click(screen.getByRole('button', { name: '复制' }));

    await waitFor(() => {
      expect(copyToClipboard).toHaveBeenLastCalledWith('editedValue');
    });

    expect(firstEntry.content_data).toBe('const answer = 42;');

    currentEntry = null;
    rerender(<DetailView />);

    expect(screen.getByText('请选择一条内容')).toBeInTheDocument();

    currentEntry = reopenedEntry;
    rerender(<DetailView />);

    expect(screen.getByTestId('renderer-session-key')).toHaveTextContent(
      'entry-code-b:hash-code-b'
    );

    fireEvent.click(screen.getByRole('button', { name: '复制' }));

    await waitFor(() => {
      expect(copyToClipboard).toHaveBeenLastCalledWith('const answer = 42;');
    });
  });
});
