import { act, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useClipboardStore } from '../../../stores/clipboardStore';
import { UrlRenderer } from './UrlRenderer';

const mockedComponents = vi.hoisted(() => ({
  unifiedTextRenderer: vi.fn(
    ({ content, contentSubType }: { content: string; contentSubType: string }) => (
      <div data-testid="unified-text-renderer">
        {contentSubType}:{content}
      </div>
    )
  ),
}));

vi.mock('../../../stores/clipboardStore', () => ({
  useClipboardStore: vi.fn(),
}));

vi.mock('./UnifiedTextRenderer', () => ({
  UnifiedTextRenderer: mockedComponents.unifiedTextRenderer,
}));

const mockedUseClipboardStore = vi.mocked(useClipboardStore);

const createDeferred = <T,>() => {
  let resolve!: (value: T) => void;

  const promise = new Promise<T>((res) => {
    resolve = res;
  });

  return { promise, resolve };
};

const createStoreState = () => ({
  copyToClipboard: vi.fn(),
  fetchUrlContent: vi.fn(),
  checkFFprobeAvailable: vi.fn().mockResolvedValue(false),
  extractMediaMetadata: vi.fn(),
});

describe('UrlRenderer', () => {
  beforeEach(() => {
    mockedUseClipboardStore.mockReturnValue(createStoreState());
  });

  it('为裸域名媒体预览使用规范化后的绝对 URL', () => {
    render(<UrlRenderer content="example.com/image.png" />);

    expect(screen.getByAltText('预览')).toHaveAttribute('src', 'https://example.com/image.png');
  });

  it('在切换 URL 条目后忽略过期的文本抓取结果', async () => {
    const firstRequest = createDeferred<string>();
    const secondRequest = createDeferred<string>();
    const store = createStoreState();

    store.fetchUrlContent.mockImplementation((url: string) => {
      if (url === 'https://example.com/first.json') {
        return firstRequest.promise;
      }

      if (url === 'https://example.com/second.json') {
        return secondRequest.promise;
      }

      return Promise.reject(new Error(`Unexpected URL: ${url}`));
    });

    mockedUseClipboardStore.mockReturnValue(store);

    const { rerender } = render(<UrlRenderer content="example.com/first.json" />);

    await waitFor(() => {
      expect(store.fetchUrlContent).toHaveBeenCalledWith('https://example.com/first.json');
    });

    rerender(<UrlRenderer content="example.com/second.json" />);

    await waitFor(() => {
      expect(store.fetchUrlContent).toHaveBeenCalledWith('https://example.com/second.json');
    });

    await act(async () => {
      secondRequest.resolve('{"id":"second"}');
      await secondRequest.promise;
    });

    await waitFor(() => {
      expect(screen.getByTestId('unified-text-renderer')).toHaveTextContent(
        /json:\s*\{\s*"id": "second"\s*\}/
      );
    });

    await act(async () => {
      firstRequest.resolve('{"id":"first"}');
      await firstRequest.promise;
    });

    expect(screen.getByTestId('unified-text-renderer')).toHaveTextContent(
      /json:\s*\{\s*"id": "second"\s*\}/
    );
  });

  it('在切换媒体 URL 时立即清空旧元数据并等待新结果', async () => {
    const secondRequest = createDeferred<{ width: number; height: number }>();
    const store = createStoreState();

    store.checkFFprobeAvailable.mockResolvedValue(true);
    store.extractMediaMetadata.mockImplementation((url: string) => {
      if (url === 'https://example.com/first.png') {
        return Promise.resolve({ width: 100, height: 50 });
      }

      if (url === 'https://example.com/second.png') {
        return secondRequest.promise;
      }

      return Promise.reject(new Error(`Unexpected URL: ${url}`));
    });

    mockedUseClipboardStore.mockReturnValue(store);

    const { rerender } = render(<UrlRenderer content="example.com/first.png" />);

    await waitFor(() => {
      expect(store.extractMediaMetadata).toHaveBeenCalledWith('https://example.com/first.png');
    });

    await waitFor(() => {
      expect(screen.getByText('100x50')).toBeInTheDocument();
    });

    rerender(<UrlRenderer content="example.com/second.png" />);

    expect(screen.queryByText('100x50')).not.toBeInTheDocument();

    await waitFor(() => {
      expect(store.extractMediaMetadata).toHaveBeenCalledWith('https://example.com/second.png');
    });

    await act(async () => {
      secondRequest.resolve({ width: 200, height: 100 });
      await secondRequest.promise;
    });

    await waitFor(() => {
      expect(screen.getByText('200x100')).toBeInTheDocument();
    });
  });
});
