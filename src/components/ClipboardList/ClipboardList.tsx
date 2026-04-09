import React, { useEffect, useCallback, useRef } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import { Card } from '../ui/card';
import { useClipboardStore } from '../../stores/clipboardStore';
import { ClipboardItem } from './ClipboardItem';
import { EmptyState } from './EmptyState';
import { RetrievalFilterBar } from './RetrievalFilterBar';

export const ClipboardList: React.FC = () => {
  const {
    loading,
    fetchHistory,
    getFilteredEntries,
    selectedEntry,
    setSelectedEntry,
    pasteSelectedEntry,
    loadMoreEntries,
    hasMore,
    isLoadingMore,
    searchTerm,
    selectedType,
    selectedSourceApp,
    favoritesOnly,
    isRetrievalActive,
    resetRetrievalFilters,
  } = useClipboardStore();
  const entries = getFilteredEntries();
  const [showNumbers, setShowNumbers] = React.useState(false);
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const retrievalActive = isRetrievalActive();
  const estimatedRowSize = retrievalActive ? 134 : 108;
  const activeFilterReasons = React.useMemo(() => {
    const reasons: string[] = [];

    if (selectedSourceApp !== 'all') {
      reasons.push('来源应用');
    }
    if (favoritesOnly) {
      reasons.push('收藏');
    }

    return reasons;
  }, [favoritesOnly, selectedSourceApp]);

  const virtualizer = useVirtualizer({
    count: entries.length,
    getScrollElement: () => scrollContainerRef.current,
    estimateSize: () => estimatedRowSize,
    overscan: 4,
  });

  useEffect(() => {
    fetchHistory();
  }, [fetchHistory]);

  useEffect(() => {
    if (entries.length === 0) {
      if (selectedEntry) {
        setSelectedEntry(null);
      }
      return;
    }

    if (!selectedEntry || !entries.some((entry) => entry.id === selectedEntry.id)) {
      setSelectedEntry(entries[0]);
    }
  }, [entries, selectedEntry, setSelectedEntry]);

  useEffect(() => {
    if (selectedEntry && entries.length > 0) {
      const selectedIndex = entries.findIndex((entry) => entry.id === selectedEntry.id);
      if (selectedIndex >= 0 && selectedIndex === 0) {
        virtualizer.scrollToIndex(0, { behavior: 'smooth' });
      }
    }
  }, [selectedEntry, entries, virtualizer]);

  useEffect(() => {
    const scrollElement = scrollContainerRef.current;
    if (!scrollElement) {
      return;
    }

    scrollElement.scrollTo({ top: 0, behavior: 'auto' });
    virtualizer.scrollToOffset(0);
  }, [favoritesOnly, searchTerm, selectedSourceApp, selectedType, virtualizer]);

  useEffect(() => {
    if (!hasMore || isLoadingMore || loading) {
      return;
    }

    const handleScroll = () => {
      const scrollElement = scrollContainerRef.current;
      if (!scrollElement) return;

      const { scrollTop, scrollHeight, clientHeight } = scrollElement;
      const scrollPercentage = (scrollTop + clientHeight) / scrollHeight;

      if (scrollPercentage > 0.9) {
        loadMoreEntries();
      }
    };

    const scrollElement = scrollContainerRef.current;
    if (scrollElement) {
      scrollElement.addEventListener('scroll', handleScroll);
      return () => scrollElement.removeEventListener('scroll', handleScroll);
    }
  }, [hasMore, isLoadingMore, loading, loadMoreEntries]);

  const scrollToSelectedEntry = useCallback(
    (index: number, direction?: 'up' | 'down') => {
      const align = direction === 'up' ? 'start' : direction === 'down' ? 'end' : 'auto';
      virtualizer.scrollToIndex(index, { align, behavior: 'smooth' });
    },
    [virtualizer]
  );

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (entries.length === 0) return;

      if (e.metaKey) {
        setShowNumbers(true);
      }

      const currentIndex = entries.findIndex((entry) => entry.id === selectedEntry?.id);

      switch (e.key) {
        case 'ArrowUp': {
          e.preventDefault();
          const prevIndex = currentIndex > 0 ? currentIndex - 1 : 0;
          setSelectedEntry(entries[prevIndex]);
          scrollToSelectedEntry(prevIndex, 'up');
          break;
        }
        case 'ArrowDown': {
          e.preventDefault();
          const nextIndex =
            currentIndex < entries.length - 1 ? currentIndex + 1 : entries.length - 1;
          setSelectedEntry(entries[nextIndex]);
          scrollToSelectedEntry(nextIndex, 'down');
          break;
        }
        case 'Meta':
          setShowNumbers(true);
          break;
        default:
          if (e.metaKey && e.key >= '1' && e.key <= '9') {
            e.preventDefault();
            const visibleItems = virtualizer.getVirtualItems();
            const visibleIndex = parseInt(e.key, 10) - 1;
            if (visibleIndex < visibleItems.length) {
              const actualIndex = visibleItems[visibleIndex].index;
              setSelectedEntry(entries[actualIndex]);
              if (pasteSelectedEntry) {
                pasteSelectedEntry(entries[actualIndex]);
              }
            }
          }
          break;
      }
    },
    [
      entries,
      selectedEntry,
      setSelectedEntry,
      pasteSelectedEntry,
      virtualizer,
      scrollToSelectedEntry,
    ]
  );

  const handleKeyUp = useCallback((e: KeyboardEvent) => {
    if (e.key === 'Meta' || !e.metaKey) {
      setShowNumbers(false);
    }
  }, []);

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('keyup', handleKeyUp);
    };
  }, [handleKeyDown, handleKeyUp]);

  useEffect(() => {
    const handleBlur = () => {
      setShowNumbers(false);
    };

    window.addEventListener('blur', handleBlur);

    return () => {
      window.removeEventListener('blur', handleBlur);
    };
  }, []);

  if (loading && entries.length === 0) {
    return (
      <Card className="flex h-full min-h-[220px] flex-col items-center justify-center rounded-[18px] border border-border/70 bg-card/85 p-4 shadow-[0_16px_50px_rgba(15,23,42,0.08)] backdrop-blur-xl min-[1200px]:min-h-[240px] min-[1200px]:rounded-[20px] min-[1200px]:p-5">
        <div className="mb-3 h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent" />
        <p className="text-sm text-muted-foreground">
          {retrievalActive ? '正在搜索剪贴板...' : '加载中...'}
        </p>
      </Card>
    );
  }

  if (entries.length === 0 && !retrievalActive) {
    return <EmptyState />;
  }

  const virtualItems = virtualizer.getVirtualItems();

  return (
    <Card
      id="clipboard-list"
      className="flex h-full min-h-0 flex-col overflow-hidden rounded-[18px] border border-border/70 bg-card/88 shadow-[0_16px_50px_rgba(15,23,42,0.08)] backdrop-blur-xl min-[1200px]:rounded-[20px]"
    >
      <RetrievalFilterBar />

      {entries.length === 0 ? (
        <div className="flex min-h-0 flex-1 p-2 min-[1200px]:p-2.5">
          <EmptyState mode="no-results" onResetFilters={resetRetrievalFilters} />
        </div>
      ) : (
        <div
          ref={scrollContainerRef}
          id="clipboard-list-scroll"
          className="flex-1 overflow-y-auto overflow-x-hidden px-1 pb-1 pt-1 min-[1200px]:px-1 min-[1200px]:pb-1 min-[1200px]:pt-1"
        >
          <div
            id="clipboard-list-items"
            className="relative"
            style={{
              height: `${virtualizer.getTotalSize() + (isLoadingMore ? 52 : 0)}px`,
            }}
          >
            {virtualItems.map((virtualItem, visibleIndex) => {
              const entry = entries[virtualItem.index];

              return (
                <div
                  key={virtualItem.key}
                  data-index={virtualItem.index}
                  ref={virtualizer.measureElement}
                  style={{
                    position: 'absolute',
                    top: 0,
                    left: 0,
                    width: '100%',
                    transform: `translateY(${virtualItem.start}px)`,
                    padding: '0 4px 5px',
                  }}
                >
                  <ClipboardItem
                    entry={entry}
                    isSelected={selectedEntry?.id === entry.id}
                    onClick={() => setSelectedEntry(entry)}
                    showNumber={showNumbers && visibleIndex + 1 <= 9}
                    number={visibleIndex + 1}
                    density={retrievalActive ? 'retrieval' : 'list'}
                    activeFilterReasons={activeFilterReasons}
                  />
                </div>
              );
            })}

            {isLoadingMore && (
              <div
                className="flex items-center justify-center py-4"
                style={{
                  position: 'absolute',
                  top: virtualizer.getTotalSize(),
                  left: 0,
                  width: '100%',
                  height: '52px',
                }}
              >
                <div className="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent" />
                <span className="ml-3 text-sm text-muted-foreground">
                  {retrievalActive ? '继续加载匹配结果...' : '加载更多...'}
                </span>
              </div>
            )}
          </div>
        </div>
      )}
    </Card>
  );
};
