import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { buildPreviewDescriptor } from '../../lib/preview/previewDescriptor';
import {
  ClipboardEntry,
  EntryAnalysisSnapshot,
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

const createAnalysis = (overrides: Partial<EntryAnalysisSnapshot>): EntryAnalysisSnapshot => ({
  contract_version: 1,
  analysis_version: 1,
  status: 'matched',
  subtype: 'plain_text',
  metadata: {
    kind: 'plain_text',
    data: {
      char_count: 11,
      line_count: 1,
    },
  },
  diagnostics: [],
  analyzed_at: new Date('2026-03-27T10:00:00Z').getTime(),
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

  it('URL 条目即使没有 resolved data 也保留 raw 和 url-structure 入口', () => {
    const descriptor = createDescriptor(
      createEntry({
        content_subtype: 'url',
        content_data: 'https://example.com/docs/guide?tab=preview',
        metadata: JSON.stringify({
          url_parts: {
            protocol: 'https',
            host: 'example.com',
            path: '/docs/guide',
            query_params: [['tab', 'preview']],
          },
        }),
        analysis: createAnalysis({
          subtype: 'url',
          metadata: {
            kind: 'url',
            data: {
              protocol: 'https',
              host: 'example.com',
              path: '/docs/guide',
              query_params: [['tab', 'preview']],
            },
          },
        }),
      })
    );

    expect(descriptor.primaryKind).toBe<PreviewKind>('url_card');
    expectAlternateKeys(descriptor, ['raw', 'url-structure']);
  });

  it('图片 URL 条目保持 URL 主预览，并暴露 resolved-image 备用视图', () => {
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

    expect(descriptor.primaryKind).toBe<PreviewKind>('url_card');
    expect(descriptor.actions).toContain('open_url');
    expectAlternateKeys(descriptor, ['raw', 'resolved-image', 'url-structure']);
  });

  it('视频 URL 条目保持 URL 主预览，并暴露 resolved-video 备用视图', () => {
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

    expect(descriptor.primaryKind).toBe<PreviewKind>('url_card');
    expectAlternateKeys(descriptor, ['raw', 'resolved-video', 'url-structure']);
  });

  it('音频 URL 条目保持 URL 主预览，并暴露 resolved-audio 备用视图', () => {
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

    expect(descriptor.primaryKind).toBe<PreviewKind>('url_card');
    expectAlternateKeys(descriptor, ['raw', 'resolved-audio', 'url-structure']);
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

  it('URL 远端 JSON 仍保持 URL 主预览，并保留 JSON 和文本备用视图', () => {
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

    expect(descriptor.primaryKind).toBe<PreviewKind>('url_card');
    expectAlternateKeys(descriptor, ['raw', 'resolved-text', 'resolved-json', 'url-structure']);
  });

  it('URL 远端文本仍保持 URL 主预览，并保留文本备用视图', () => {
    const descriptor = createDescriptor(
      createEntry({
        content_subtype: 'url',
        content_data: 'https://example.com/readme.txt',
      }),
      {
        textContent: '# hello from remote',
        url: { previewKind: 'plain_text' },
      }
    );

    expect(descriptor.primaryKind).toBe<PreviewKind>('url_card');
    expectAlternateKeys(descriptor, ['raw', 'resolved-text', 'url-structure']);
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

  it('analysis-first 会覆盖 legacy subtype 和 metadata', () => {
    const descriptor = createDescriptor(
      createEntry({
        content_subtype: 'json',
        content_data: 'https://analysis.example.com/docs?tab=1',
        metadata: JSON.stringify({
          url_parts: {
            protocol: 'https',
            host: 'legacy.example.com',
            path: '/wrong',
            query_params: [['mode', 'legacy']],
          },
        }),
        analysis: createAnalysis({
          subtype: 'url',
          metadata: {
            kind: 'url',
            data: {
              protocol: 'https',
              host: 'analysis.example.com',
              path: '/docs',
              query_params: [{ key: 'tab', value: '1' }],
            },
          },
        }),
      })
    );

    expect(descriptor.primaryKind).toBe<PreviewKind>('url_card');
    expect(descriptor.actions).toContain('open_url');
    expect(descriptor.inspectorSections.find((section) => section.title === 'URL')?.items).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ label: 'Host', value: 'analysis.example.com' }),
        expect.objectContaining({ label: 'Path', value: '/docs' }),
      ])
    );
  });

  it('fallback analysis 会保留 raw 主视图并暴露 diagnostics inspector', () => {
    const descriptor = createDescriptor(
      createEntry({
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
          metadata: {
            kind: 'plain_text',
            data: {
              char_count: 12,
              line_count: 1,
            },
          },
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

    expect(descriptor.primaryKind).toBe<PreviewKind>('plain_text');
    expect(descriptor.badges).toEqual(
      expect.arrayContaining([expect.objectContaining({ label: 'Fallback', tone: 'warning' })])
    );
    expectAlternateKeys(descriptor, ['raw']);
    expect(
      descriptor.inspectorSections.find((section) => section.title === 'Analysis')?.items
    ).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ label: 'Status', value: 'Fallback' }),
        expect.objectContaining({ value: expect.stringContaining('json_malformed') }),
      ])
    );
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
