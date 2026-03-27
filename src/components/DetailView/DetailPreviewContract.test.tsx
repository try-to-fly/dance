import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { buildPreviewDescriptor } from '../../lib/preview/previewDescriptor';
import {
  ClipboardEntry,
  PreviewDescriptor,
  PreviewKind,
  ResolvedPreviewData,
} from '../../types/clipboard';
import { DetailEmptyState, DetailScene } from './scene/DetailScene';

vi.mock('./scene/PrimaryPreviewRenderer', () => ({
  PrimaryPreviewRenderer: () => <div data-testid="primary-preview-renderer" />,
}));

vi.mock('./scene/AlternateViews', () => ({
  AlternateViews: ({ views }: { views: Array<{ key: string }> }) => (
    <div data-testid="alternate-views">{views.map((view) => view.key).join(',')}</div>
  ),
}));

vi.mock('./scene/InspectorPanel', () => ({
  InspectorPanel: ({ sections }: { sections: Array<{ title: string }> }) => (
    <div data-testid="inspector-panel">{sections.map((section) => section.title).join(',')}</div>
  ),
}));

const previewLabels = {
  unknown: 'Unknown',
  image: 'Image',
  file: 'File',
  text: 'Text',
  base64: 'Base64',
  subtypeLabels: {
    plain_text: 'Text',
    url: 'URL',
    ip_address: 'IP',
    email: 'Email',
    color: 'Color',
    code: 'Code',
    command: 'Command',
    timestamp: 'Timestamp',
    json: 'JSON',
    markdown: 'Markdown',
    base64: 'Base64',
  },
} as const;

const baseEntry: ClipboardEntry = {
  id: 'entry-1',
  content_hash: 'hash-1',
  content_type: 'text/plain',
  content_data: 'raw-content',
  source_app: 'Terminal',
  created_at: new Date('2026-03-27T10:00:00Z').getTime(),
  copy_count: 1,
  file_path: null,
  is_favorite: false,
  content_subtype: 'plain_text',
  metadata: null,
  app_bundle_id: null,
};

const createEntry = (overrides: Partial<ClipboardEntry>): ClipboardEntry => ({
  ...baseEntry,
  ...overrides,
});

const createDescriptor = (entry: ClipboardEntry, resolvedData?: ResolvedPreviewData) =>
  buildPreviewDescriptor({
    entry,
    resolvedData,
    labels: previewLabels,
  });

const expectAlternateKeys = (descriptor: PreviewDescriptor, expected: string[]) => {
  expect(descriptor.alternateViews.map((view) => view.key)).toEqual(expected);
};

describe('DetailPreview 契约 - Descriptor', () => {
  it('JSON 条目默认进入结构化主预览并保留 raw 备用视图', () => {
    const descriptor = createDescriptor(
      createEntry({
        content_subtype: 'json',
        content_data: '{"name":"dance"}',
      })
    );

    expect(descriptor.primaryKind).toBe<PreviewKind>('json');
    expectAlternateKeys(descriptor, ['raw']);
  });

  it('图片 URL 条目优先进入图片主预览', () => {
    const descriptor = createDescriptor(
      createEntry({
        content_subtype: 'url',
        content_data: 'https://example.com/a.png',
        metadata: JSON.stringify({
          url_parts: {
            protocol: 'https',
            host: 'example.com',
            path: '/a.png',
            query_params: [],
          },
        }),
      }),
      {
        imageUrl: 'https://example.com/a.png',
        url: { previewKind: 'image' },
      }
    );

    expect(descriptor.primaryKind).toBe<PreviewKind>('image');
    expect(descriptor.actions).toContain('open_url');
    expectAlternateKeys(descriptor, ['raw', 'url-structure']);
  });

  it('视频 URL 条目优先进入视频主预览', () => {
    const descriptor = createDescriptor(
      createEntry({
        content_subtype: 'url',
        content_data: 'https://example.com/a.mp4',
      }),
      {
        videoUrl: 'https://example.com/a.mp4',
        url: { previewKind: 'video' },
      }
    );

    expect(descriptor.primaryKind).toBe<PreviewKind>('video');
    expectAlternateKeys(descriptor, ['raw', 'url-structure']);
  });

  it('音频 URL 条目优先进入音频主预览', () => {
    const descriptor = createDescriptor(
      createEntry({
        content_subtype: 'url',
        content_data: 'https://example.com/a.mp3',
      }),
      {
        audioUrl: 'https://example.com/a.mp3',
        url: { previewKind: 'audio' },
      }
    );

    expect(descriptor.primaryKind).toBe<PreviewKind>('audio');
    expectAlternateKeys(descriptor, ['raw', 'url-structure']);
  });

  it('Base64 JSON 条目优先进入 JSON 主预览并包含 decoded 备用视图', () => {
    const descriptor = createDescriptor(
      createEntry({
        content_subtype: 'base64',
        content_data: 'eyJuYW1lIjoiZGFuY2UifQ==',
        metadata: JSON.stringify({
          base64_metadata: { encoded_size: 20, estimated_original_size: 14 },
        }),
      }),
      {
        jsonContent: { name: 'dance' },
        base64: {
          decodedKind: 'json',
          textPreview: '{"name":"dance"}',
          mime: 'application/json',
          sizeBytes: 14,
        },
      }
    );

    expect(descriptor.primaryKind).toBe<PreviewKind>('json');
    expect(descriptor.actions).toContain('copy_decoded');
    expectAlternateKeys(descriptor, ['raw', 'resolved-json', 'decoded']);
  });

  it('JSON 合法值 null 仍然进入 JSON 主预览并保留结构化备用视图', () => {
    const descriptor = createDescriptor(
      createEntry({
        content_subtype: 'url',
        content_data: 'https://example.com/null.json',
      }),
      {
        textContent: 'null',
        jsonContent: null,
        url: { previewKind: 'json' },
      }
    );

    expect(descriptor.primaryKind).toBe<PreviewKind>('json');
    expectAlternateKeys(descriptor, ['raw', 'resolved-text', 'resolved-json', 'url-structure']);
  });

  it('Base64 图片条目优先进入图片主预览', () => {
    const descriptor = createDescriptor(
      createEntry({
        content_subtype: 'base64',
        content_data: 'iVBORw0KGgoAAAANSUhEUgAAAAUA',
      }),
      {
        imageUrl: 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAUA',
        base64: {
          decodedKind: 'image',
          mime: 'image/png',
          dataUrl: 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAUA',
          sizeBytes: 128,
        },
      }
    );

    expect(descriptor.primaryKind).toBe<PreviewKind>('image');
    expect(descriptor.actions).not.toContain('copy_decoded');
    expectAlternateKeys(descriptor, ['raw']);
  });

  it('缺少 data URL 的 Base64 媒体会一致降级到二进制卡片', () => {
    const descriptor = createDescriptor(
      createEntry({
        content_subtype: 'base64',
        content_data: 'iVBORw0KGgoAAAANSUhEUgAAAAUA',
      }),
      {
        base64: {
          decodedKind: 'image',
          mime: 'image/png',
          sizeBytes: 5 * 1024 * 1024,
          error: 'Decoded media is too large to inline as data URL',
        },
      }
    );

    expect(descriptor.primaryKind).toBe<PreviewKind>('base64_binary');
    expect(descriptor.actions).not.toContain('copy_decoded');
  });

  it('普通 URL 在未解析出目标内容时回退 URL 卡片', () => {
    const descriptor = createDescriptor(
      createEntry({
        content_subtype: 'url',
        content_data: 'https://example.com/docs',
      })
    );

    expect(descriptor.primaryKind).toBe<PreviewKind>('url_card');
    expectAlternateKeys(descriptor, ['raw', 'url-structure']);
  });
});

describe('DetailPreview 契约 - Scene', () => {
  it('DetailEmptyState 作为空状态入口可渲染', () => {
    render(<DetailEmptyState selectItemLabel="请选择内容" />);
    expect(screen.getByText('请选择内容')).toBeInTheDocument();
  });

  it('DetailScene 在有 inspector 数据时渲染 inspector 并触发操作', () => {
    const descriptor: PreviewDescriptor = {
      headline: 'example.com/a.png',
      typeLabel: 'URL',
      badges: [],
      primaryKind: 'image',
      primaryPayload: {},
      inspectorSections: [{ title: 'URL', items: [{ label: 'Host', value: 'example.com' }] }],
      alternateViews: [{ key: 'raw', label: 'Raw', kind: 'raw', payload: 'raw-content' }],
      actions: ['copy_raw', 'paste', 'open_url'],
    };

    const onCopy = vi.fn();
    const onPaste = vi.fn();
    const onDelete = vi.fn();
    const onToggleFavorite = vi.fn();
    const onOpenUrl = vi.fn();
    const onOpenFile = vi.fn();

    render(
      <DetailScene
        entry={createEntry({ content_subtype: 'url', content_data: 'https://example.com/a.png' })}
        descriptor={descriptor}
        metadataPills={[
          {
            key: 'source',
            icon: () => <span data-testid="meta-icon" />,
            label: '来源',
            value: 'Terminal',
            fullValue: 'Terminal',
          },
        ]}
        labels={{
          copy: '复制',
          copyDecoded: '复制解析内容',
          paste: '粘贴',
          delete: '删除',
          favorite: '收藏',
          unfavorite: '取消收藏',
          openFile: '打开文件',
          openUrl: '打开链接',
          title: '详情预览',
        }}
        onCopy={onCopy}
        onCopyDecoded={vi.fn()}
        onPaste={onPaste}
        onDelete={onDelete}
        onToggleFavorite={onToggleFavorite}
        onOpenUrl={onOpenUrl}
        onOpenFile={onOpenFile}
      />
    );

    expect(screen.getByTestId('primary-preview-renderer')).toBeInTheDocument();
    expect(screen.queryByTestId('alternate-views')).not.toBeInTheDocument();
    expect(screen.getByTestId('inspector-panel')).toHaveTextContent('URL');

    fireEvent.click(screen.getByRole('button', { name: '复制' }));
    fireEvent.click(screen.getByRole('button', { name: '粘贴' }));
    fireEvent.click(screen.getByRole('button', { name: '删除' }));

    expect(onCopy).toHaveBeenCalledTimes(1);
    expect(onPaste).toHaveBeenCalledTimes(1);
    expect(onDelete).toHaveBeenCalledTimes(1);
  });

  it('DetailScene 只渲染 descriptor.actions 中声明的上下文动作', () => {
    render(
      <DetailScene
        entry={createEntry({ content_subtype: 'url', content_data: 'https://example.com/file' })}
        descriptor={{
          headline: 'example.com/file',
          typeLabel: 'URL',
          badges: [],
          primaryKind: 'url_card',
          primaryPayload: {},
          inspectorSections: [],
          alternateViews: [],
          actions: ['open_url', 'copy_decoded'],
        }}
        metadataPills={[]}
        labels={{
          copy: '复制',
          copyDecoded: '复制解析内容',
          paste: '粘贴',
          delete: '删除',
          favorite: '收藏',
          unfavorite: '取消收藏',
          openFile: '打开文件',
          openUrl: '打开链接',
          title: '详情预览',
        }}
        onCopy={vi.fn()}
        onCopyDecoded={vi.fn()}
        onPaste={vi.fn()}
        onDelete={vi.fn()}
        onToggleFavorite={vi.fn()}
        onOpenUrl={vi.fn()}
        onOpenFile={vi.fn()}
      />
    );

    expect(screen.getByRole('button', { name: '打开链接' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '复制解析内容' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: '复制' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: '粘贴' })).not.toBeInTheDocument();
  });

  it('沉浸式主预览会进入紧凑布局并隐藏单个 raw 备用视图', () => {
    const descriptor: PreviewDescriptor = {
      headline: 'image.png',
      typeLabel: '图片',
      badges: [],
      primaryKind: 'image',
      primaryPayload: {},
      inspectorSections: [],
      alternateViews: [{ key: 'raw', label: 'Raw', kind: 'raw', payload: 'ignored-raw' }],
      actions: ['paste'],
    };

    render(
      <DetailScene
        entry={createEntry({
          content_type: 'image/png',
          content_data: null,
          file_path: '/tmp/a.png',
        })}
        descriptor={descriptor}
        metadataPills={[]}
        labels={{
          copy: '复制',
          copyDecoded: '复制解析内容',
          paste: '粘贴',
          delete: '删除',
          favorite: '收藏',
          unfavorite: '取消收藏',
          openFile: '打开文件',
          openUrl: '打开链接',
          title: '详情预览',
        }}
        onCopy={vi.fn()}
        onCopyDecoded={vi.fn()}
        onPaste={vi.fn()}
        onDelete={vi.fn()}
        onToggleFavorite={vi.fn()}
        onOpenUrl={vi.fn()}
        onOpenFile={vi.fn()}
      />
    );

    expect(document.getElementById('detail-view')).toHaveAttribute('data-layout', 'immersive');
    expect(screen.queryByTestId('alternate-views')).not.toBeInTheDocument();
  });
});
