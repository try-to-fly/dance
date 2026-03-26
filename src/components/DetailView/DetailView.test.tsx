import { render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useClipboardStore } from '../../stores/clipboardStore';
import { ClipboardEntry } from '../../types/clipboard';
import { DetailView } from './DetailView';

const mockedComponents = vi.hoisted(() => ({
  unifiedTextRenderer: vi.fn(
    ({ content, contentSubType }: { content: string; contentSubType: string }) => (
      <div data-testid="renderer-unified">
        {contentSubType}:{content}
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
}));

vi.mock('../../stores/clipboardStore', () => ({
  useClipboardStore: vi.fn(),
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
        'detail.contentTypes.command': '命令',
        'detail.contentTypes.json': 'JSON',
        'detail.contentTypes.image': '图片',
        'detail.contentTypes.file': '文件',
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

const createStoreState = (selectedEntry: ClipboardEntry | null) => ({
  selectedEntry,
  getImageUrl: vi.fn().mockResolvedValue('tauri://image-preview'),
  openFileWithSystem: vi.fn(),
  copyToClipboard: vi.fn(),
  pasteSelectedEntry: vi.fn(),
  toggleFavorite: vi.fn(),
  deleteEntry: vi.fn(),
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
  beforeEach(() => {
    mockedUseClipboardStore.mockReturnValue(createStoreState(null));
  });

  it('在未选中内容时展示空状态', () => {
    render(<DetailView />);

    expect(screen.getByText('请选择一条内容')).toBeInTheDocument();
  });

  it('将类型放到紧凑标签中，并为来源/时间/次数保留压缩元信息展示', () => {
    mockedUseClipboardStore.mockReturnValue(
      createStoreState({
        ...baseEntry,
        content_data: 'npm run dev',
        copy_count: 12,
      })
    );

    render(<DetailView />);

    expect(screen.getByTestId('renderer-unified')).toHaveTextContent('command:npm run dev');
    expect(document.getElementById('detail-view-type-badge')).toHaveTextContent('命令');
    expect(document.getElementById('detail-view-title')).toHaveTextContent('npm run dev');
    expect(document.getElementById('detail-view-metadata')).toHaveClass('flex', 'flex-wrap');
    expect(document.getElementById('detail-view-metadata')?.children).toHaveLength(3);
    expect(screen.getByText('来源')).toBeInTheDocument();
    expect(screen.getByText('时间')).toBeInTheDocument();
    expect(screen.getByText('复制次数')).toBeInTheDocument();
    expect(screen.getByText('Terminal')).toBeInTheDocument();
    expect(screen.getByText(formatTimestamp(baseEntry.created_at, true))).toBeInTheDocument();
    expect(screen.getByText('12')).toBeInTheDocument();
    expect(screen.queryByText('类型')).not.toBeInTheDocument();
  });

  it('根据内容子类型切换到 JSON 渲染器', () => {
    mockedUseClipboardStore.mockReturnValue(
      createStoreState({
        ...baseEntry,
        content_subtype: 'json',
        content_data: '{"hello":"world"}',
      })
    );

    render(<DetailView />);

    expect(screen.getByTestId('renderer-json')).toHaveTextContent('{"hello":"world"}');
    expect(screen.queryByTestId('renderer-unified')).not.toBeInTheDocument();
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
});
