import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useVirtualizer } from '@tanstack/react-virtual';
import { useClipboardStore } from '../../stores/clipboardStore';
import type { ClipboardEntry } from '../../types/clipboard';
import { ClipboardList } from './ClipboardList';

vi.mock('../../stores/clipboardStore', () => ({
  useClipboardStore: vi.fn(),
}));

vi.mock('@tanstack/react-virtual', () => ({
  useVirtualizer: vi.fn(),
}));

vi.mock('./ClipboardItem', () => ({
  ClipboardItem: ({
    entry,
    showNumber,
    number,
  }: {
    entry: ClipboardEntry;
    showNumber?: boolean;
    number?: number;
  }) => (
    <div
      data-testid={`clipboard-item-${entry.id}`}
      data-show-number={showNumber ? 'true' : 'false'}
      data-number={number ?? ''}
    >
      {showNumber ? <span>{number}</span> : null}
      <span>{entry.content_data}</span>
    </div>
  ),
}));

vi.mock('./EmptyState', () => ({
  EmptyState: () => <div>empty</div>,
}));

vi.mock('./RetrievalFilterBar', () => ({
  RetrievalFilterBar: () => <div>filters</div>,
}));

const mockedUseClipboardStore = vi.mocked(useClipboardStore);
const mockedUseVirtualizer = vi.mocked(useVirtualizer);

const scrollToIndex = vi.fn();
const scrollToOffset = vi.fn();
const measureElement = vi.fn();

const entries: ClipboardEntry[] = [
  {
    id: 'entry-1',
    content_hash: 'hash-1',
    content_type: 'text/plain',
    content_data: 'first entry',
    source_app: 'Terminal',
    created_at: new Date('2026-03-28T10:00:00Z').getTime(),
    copy_count: 1,
    file_path: null,
    is_favorite: false,
    content_subtype: 'plain_text',
    metadata: null,
    app_bundle_id: null,
    analysis: null,
  },
  {
    id: 'entry-2',
    content_hash: 'hash-2',
    content_type: 'text/plain',
    content_data: 'second entry',
    source_app: 'Terminal',
    created_at: new Date('2026-03-28T10:01:00Z').getTime(),
    copy_count: 1,
    file_path: null,
    is_favorite: false,
    content_subtype: 'plain_text',
    metadata: null,
    app_bundle_id: null,
    analysis: null,
  },
];

const createStoreState = () => ({
  loading: false,
  fetchHistory: vi.fn(),
  getFilteredEntries: () => entries,
  selectedEntry: entries[0],
  setSelectedEntry: vi.fn(),
  pasteSelectedEntry: vi.fn(),
  loadMoreEntries: vi.fn(),
  hasMore: false,
  isLoadingMore: false,
  searchTerm: '',
  selectedType: 'all',
  selectedSourceApp: 'all',
  favoritesOnly: false,
  isRetrievalActive: vi.fn(() => false),
  resetRetrievalFilters: vi.fn(),
});

describe('ClipboardList', () => {
  beforeEach(() => {
    scrollToIndex.mockReset();
    scrollToOffset.mockReset();
    measureElement.mockReset();

    Object.defineProperty(HTMLElement.prototype, 'scrollTo', {
      configurable: true,
      writable: true,
      value: vi.fn(),
    });

    mockedUseVirtualizer.mockReturnValue({
      getVirtualItems: () =>
        entries.map((entry, index) => ({
          key: entry.id,
          index,
          start: index * 100,
        })),
      getTotalSize: () => entries.length * 100,
      scrollToIndex,
      scrollToOffset,
      measureElement,
    } as unknown as ReturnType<typeof useVirtualizer>);
  });

  it('按下 Command 时展示数字角标，并通过 Command+数字触发快速粘贴', () => {
    const storeState = createStoreState();
    mockedUseClipboardStore.mockReturnValue(storeState);

    render(<ClipboardList />);

    expect(screen.getByTestId('clipboard-item-entry-1')).toHaveAttribute(
      'data-show-number',
      'false'
    );
    expect(screen.getByTestId('clipboard-item-entry-2')).toHaveAttribute(
      'data-show-number',
      'false'
    );

    fireEvent.keyDown(window, { key: 'Meta', metaKey: true });

    expect(screen.getByTestId('clipboard-item-entry-1')).toHaveAttribute(
      'data-show-number',
      'true'
    );
    expect(screen.getByTestId('clipboard-item-entry-2')).toHaveAttribute(
      'data-show-number',
      'true'
    );

    fireEvent.keyDown(window, { key: '2', metaKey: true });

    expect(storeState.setSelectedEntry).toHaveBeenCalledWith(entries[1]);
    expect(storeState.pasteSelectedEntry).toHaveBeenCalledWith(entries[1]);

    fireEvent.keyUp(window, { key: 'Meta' });

    expect(screen.getByTestId('clipboard-item-entry-1')).toHaveAttribute(
      'data-show-number',
      'false'
    );
    expect(screen.getByTestId('clipboard-item-entry-2')).toHaveAttribute(
      'data-show-number',
      'false'
    );
  });

  it('按下 Option 不再展示数字角标，也不会触发快速粘贴', () => {
    const storeState = createStoreState();
    mockedUseClipboardStore.mockReturnValue(storeState);

    render(<ClipboardList />);

    fireEvent.keyDown(window, { key: 'Alt', altKey: true });

    expect(screen.getByTestId('clipboard-item-entry-1')).toHaveAttribute(
      'data-show-number',
      'false'
    );
    expect(screen.getByTestId('clipboard-item-entry-2')).toHaveAttribute(
      'data-show-number',
      'false'
    );

    fireEvent.keyDown(window, { key: '1', altKey: true });

    expect(storeState.setSelectedEntry).not.toHaveBeenCalled();
    expect(storeState.pasteSelectedEntry).not.toHaveBeenCalled();
  });
});
