import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ClipboardEntry } from '../../../types/clipboard';
import { PrimaryPreviewRenderer } from './PrimaryPreviewRenderer';

const rendererMocks = vi.hoisted(() => ({
  jsonRenderer: vi.fn(({ content }: { content: string }) => (
    <div data-testid="json-renderer">{content}</div>
  )),
  unifiedTextRenderer: vi.fn(({ content }: { content: string }) => (
    <div data-testid="text-renderer">{content}</div>
  )),
  colorRenderer: vi.fn(({ content }: { content: string }) => (
    <div data-testid="color-renderer">{content}</div>
  )),
  ipRenderer: vi.fn(({ content }: { content: string }) => (
    <div data-testid="ip-renderer">{content}</div>
  )),
  emailRenderer: vi.fn(({ content }: { content: string }) => (
    <div data-testid="email-renderer">{content}</div>
  )),
  timeRenderer: vi.fn(({ content }: { content: string }) => (
    <div data-testid="time-renderer">{content}</div>
  )),
  imagePreview: vi.fn(({ imageUrl }: { imageUrl: string }) => (
    <div data-testid="image-preview">{imageUrl}</div>
  )),
}));

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, options?: { defaultValue?: string }) => {
      const dictionary: Record<string, string> = {
        'detail.contentTypes.url': 'URL',
        'detail.contentTypes.video': 'Video',
        'detail.contentTypes.audio': 'Audio',
        'detail.contentTypes.base64': 'Base64',
        'detail.unknown': 'Unknown',
      };

      return dictionary[key] ?? options?.defaultValue ?? key;
    },
  }),
}));

vi.mock('../ContentRenderers', () => ({
  JsonRenderer: rendererMocks.jsonRenderer,
  UnifiedTextRenderer: rendererMocks.unifiedTextRenderer,
  ColorRenderer: rendererMocks.colorRenderer,
  IpRenderer: rendererMocks.ipRenderer,
  EmailRenderer: rendererMocks.emailRenderer,
  TimeRenderer: rendererMocks.timeRenderer,
}));

vi.mock('../ImagePreview', () => ({
  ImagePreview: rendererMocks.imagePreview,
}));

const baseEntry: ClipboardEntry = {
  id: 'entry-1',
  content_hash: 'hash-1',
  content_type: 'text/plain',
  content_data: 'https://example.com/docs?tab=preview&lang=zh',
  source_app: 'Terminal',
  created_at: Date.now(),
  copy_count: 1,
  file_path: null,
  is_favorite: false,
  content_subtype: 'url',
  metadata: null,
  app_bundle_id: null,
};

describe('PrimaryPreviewRenderer', () => {
  it('kind=url_card 时展示本地 URL 结构卡，而不是 raw code block', () => {
    render(
      <PrimaryPreviewRenderer
        kind="url_card"
        payload={{
          entry: baseEntry,
          subType: 'url',
          metadata: {
            url_parts: {
              protocol: 'https',
              host: 'example.com',
              path: '/docs',
              query_params: [
                ['tab', 'preview'],
                ['lang', 'zh'],
              ],
            },
          },
        }}
        onOpenFile={vi.fn()}
      />
    );

    expect(screen.getByText('Protocol')).toBeInTheDocument();
    expect(screen.getByText('https')).toBeInTheDocument();
    expect(screen.getByText('Host')).toBeInTheDocument();
    expect(screen.getByText('example.com')).toBeInTheDocument();
    expect(screen.getByText('Path')).toBeInTheDocument();
    expect(screen.getByText('/docs')).toBeInTheDocument();
    expect(screen.getByText('Query')).toBeInTheDocument();
    expect(screen.getByText('tab')).toBeInTheDocument();
    expect(screen.getByText('preview')).toBeInTheDocument();
  });

  it('JSON primary renderer 回归时继续走 JsonRenderer', () => {
    render(
      <PrimaryPreviewRenderer
        kind="json"
        payload={{
          entry: {
            ...baseEntry,
            content_subtype: 'json',
            content_data: '{"name":"dance"}',
          },
          subType: 'json',
          metadata: null,
        }}
        onOpenFile={vi.fn()}
      />
    );

    expect(screen.getByTestId('json-renderer')).toHaveTextContent('{"name":"dance"}');
    expect(screen.queryByTestId('text-renderer')).not.toBeInTheDocument();
  });
});
