import { render, screen } from '@testing-library/react';
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
  it('只有 raw 单一备用视图时不渲染容器', () => {
    const { container } = render(
      <AlternateViews views={[{ key: 'raw', label: 'Raw', kind: 'raw', payload: 'hello' }]} />
    );

    expect(container).toBeEmptyDOMElement();
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
});
