import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { AlternateViews } from './AlternateViews';

vi.mock('../ContentRenderers', () => ({
  JsonRenderer: ({ content }: { content: string }) => (
    <div data-testid="json-renderer">{content}</div>
  ),
  UnifiedTextRenderer: ({ content }: { content: string }) => (
    <div data-testid="text-renderer">{content}</div>
  ),
}));

describe('AlternateViews', () => {
  it('只有 raw 单一备用视图时仍渲染 Raw 卡片', () => {
    render(
      <AlternateViews views={[{ key: 'raw', label: 'Raw', kind: 'raw', payload: 'hello' }]} />
    );

    expect(screen.getByText('Raw')).toBeInTheDocument();
    expect(screen.getByTestId('text-renderer')).toHaveTextContent('hello');
  });

  it('存在多个备用视图时保留 tabs 容器', () => {
    render(
      <AlternateViews
        views={[
          { key: 'raw', label: 'Raw', kind: 'raw', payload: 'hello' },
          { key: 'decoded', label: 'Decoded', kind: 'plain_text', payload: 'world' },
        ]}
      />
    );

    expect(screen.getByRole('tab', { name: 'Raw' })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: 'Decoded' })).toBeInTheDocument();
  });

  it('普通 detail 布局下 raw 和 url-structure 视图都稳定可达', () => {
    render(
      <AlternateViews
        views={[
          {
            key: 'raw',
            label: 'Raw',
            kind: 'raw',
            payload: 'https://example.com/docs?tab=preview&lang=zh',
          },
          {
            key: 'url-structure',
            label: 'URL',
            kind: 'url_card',
            payload: {
              raw: 'https://example.com/docs?tab=preview&lang=zh',
              parts: {
                protocol: 'https',
                host: 'example.com',
                path: '/docs',
                query_params: [
                  ['tab', 'preview'],
                  ['lang', 'zh'],
                ],
              },
            },
          },
        ]}
      />
    );

    expect(screen.getByRole('tab', { name: 'Raw' })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: 'URL' })).toBeInTheDocument();
    expect(screen.getByTestId('text-renderer')).toHaveTextContent(
      'https://example.com/docs?tab=preview&lang=zh'
    );

    fireEvent.click(screen.getByRole('tab', { name: 'URL' }));

    expect(screen.getByText('Protocol')).toBeInTheDocument();
    expect(screen.getByText('Host')).toBeInTheDocument();
    expect(screen.getByText('Path')).toBeInTheDocument();
    expect(screen.getByText('Query')).toBeInTheDocument();
  });

  it('JSON 备用视图走 JsonRenderer，而不是统一文本渲染器', () => {
    render(
      <AlternateViews
        views={[{ key: 'resolved-json', label: 'JSON', kind: 'json', payload: '{"ok":true}' }]}
      />
    );

    expect(screen.getByTestId('json-renderer')).toHaveTextContent('{"ok":true}');
    expect(screen.queryByTestId('text-renderer')).not.toBeInTheDocument();
  });

  it('resolved image 备用视图渲染原生图片预览', () => {
    render(
      <AlternateViews
        views={[
          {
            key: 'resolved-image',
            label: 'Image',
            kind: 'image',
            payload: 'https://example.com/preview.png',
          },
        ]}
      />
    );

    expect(screen.getByAltText('Image preview')).toHaveAttribute(
      'src',
      'https://example.com/preview.png'
    );
    expect(screen.queryByTestId('text-renderer')).not.toBeInTheDocument();
  });

  it('resolved audio 备用视图渲染原生音频播放器', () => {
    render(
      <AlternateViews
        views={[
          {
            key: 'resolved-audio',
            label: 'Audio',
            kind: 'audio',
            payload: 'https://example.com/preview.mp3',
          },
        ]}
      />
    );

    expect(screen.getByLabelText('Audio preview')).toHaveAttribute(
      'src',
      'https://example.com/preview.mp3'
    );
    expect(screen.queryByTestId('text-renderer')).not.toBeInTheDocument();
  });

  it('resolved video 备用视图渲染原生视频播放器', () => {
    render(
      <AlternateViews
        views={[
          {
            key: 'resolved-video',
            label: 'Video',
            kind: 'video',
            payload: 'https://example.com/preview.mp4',
          },
        ]}
      />
    );

    expect(screen.getByLabelText('Video preview')).toHaveAttribute(
      'src',
      'https://example.com/preview.mp4'
    );
    expect(screen.queryByTestId('text-renderer')).not.toBeInTheDocument();
  });
});
