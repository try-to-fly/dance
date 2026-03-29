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
    recencyDays,
    isRetrievalActive,
    resetRetrievalFilters,
  } = useClipboardStore();
  const entries = getFilteredEntries();
  const [showNumbers, setShowNumbers] = React.useState(false);
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const retrievalActive = isRetrievalActive();
  const estimatedRowSize = retrievalActive ? 172 : 140;
  const activeFilterReasons = React.useMemo(() => {
    const reasons: string[] = [];

    if (selectedSourceApp !== 'all') {
      reasons.push('来源应用');
    }
    if (favoritesOnly) {
      reasons.push('收藏');
    }
    if (recencyDays === 1) {
      reasons.push('最近 24 小时');
    }
    if (recencyDays === 7) {
      reasons.push('最近 7 天');
    }
    if (recencyDays === 30) {
      reasons.push('最近 30 天');
    }

    return reasons;
  }, [favoritesOnly, recencyDays, selectedSourceApp]);

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
  }, [favoritesOnly, recencyDays, searchTerm, selectedSourceApp, selectedType, virtualizer]);

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
        case 'Alt':
          setShowNumbers(true);
          break;
        default:
          if (e.altKey && e.key >= '1' && e.key <= '9') {
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
    if (e.key === 'Alt') {
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

  if (loading && entries.length === 0) {
    return (
      <Card className="flex h-full min-h-[280px] flex-col items-center justify-center rounded-[22px] border border-border/70 bg-card/85 p-6 shadow-[0_16px_50px_rgba(15,23,42,0.08)] backdrop-blur-xl min-[1200px]:min-h-[320px] min-[1200px]:rounded-[26px] min-[1200px]:p-8">
        <div className="mb-4 h-9 w-9 animate-spin rounded-full border-2 border-primary border-t-transparent" />
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
      className="flex h-full min-h-0 flex-col overflow-hidden rounded-[22px] border border-border/70 bg-card/88 shadow-[0_16px_50px_rgba(15,23,42,0.08)] backdrop-blur-xl min-[1200px]:rounded-[26px]"
    >
      <RetrievalFilterBar />

      {entries.length === 0 ? (
        <div className="flex min-h-0 flex-1 p-3 min-[1200px]:p-3.5">
          <EmptyState mode="no-results" onResetFilters={resetRetrievalFilters} />
        </div>
      ) : (
        <div
          ref={scrollContainerRef}
          id="clipboard-list-scroll"
          className="flex-1 overflow-y-auto overflow-x-hidden px-1.5 pb-1 pt-1.5 min-[1200px]:px-2 min-[1200px]:pb-1.5 min-[1200px]:pt-2"
        >
          <div
            id="clipboard-list-items"
            className="relative"
            style={{
              height: `${virtualizer.getTotalSize() + (isLoadingMore ? 68 : 0)}px`,
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
                    padding: '0 6px 8px',
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
                  height: '68px',
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
