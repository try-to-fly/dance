import React, { useDeferredValue, useEffect, useState } from 'react';
import { Search, X } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { Input } from '../ui/input';
import { Button } from '../ui/button';
import { useClipboardStore } from '../../stores/clipboardStore';
import { analytics, ANALYTICS_EVENTS } from '../../services/analytics';
import { cn } from '../../lib/utils';

interface SearchBarProps {
  compact?: boolean;
  className?: string;
}

export const SearchBar: React.FC<SearchBarProps> = ({ compact = false, className }) => {
  const { t } = useTranslation(['common', 'clipboard']);
  const { searchTerm, setSearchTerm, selectedType, selectedSourceApp, favoritesOnly, loading } =
    useClipboardStore();
  const [localSearchTerm, setLocalSearchTerm] = useState(searchTerm);
  const deferredSearchTerm = useDeferredValue(localSearchTerm);
  const [showPendingIndicator, setShowPendingIndicator] = useState(false);
  const hasActiveFilters = selectedType !== 'all' || selectedSourceApp !== 'all' || favoritesOnly;
  const isRetrievalActive = Boolean(searchTerm.trim()) || hasActiveFilters;

  useEffect(() => {
    setLocalSearchTerm(searchTerm);
  }, [searchTerm]);

  useEffect(() => {
    const debounceTimer = window.setTimeout(() => {
      setSearchTerm(deferredSearchTerm);
    }, 200);

    return () => window.clearTimeout(debounceTimer);
  }, [deferredSearchTerm, setSearchTerm]);

  useEffect(() => {
    if (!deferredSearchTerm.trim()) {
      return;
    }

    const analyticsTimer = window.setTimeout(() => {
      analytics.track(ANALYTICS_EVENTS.SEARCH_PERFORMED, {
        has_filter: hasActiveFilters ? 1 : 0,
        filter_type: selectedType,
      });
    }, 200);

    return () => window.clearTimeout(analyticsTimer);
  }, [deferredSearchTerm, hasActiveFilters, selectedType]);

  useEffect(() => {
    if (!loading || !isRetrievalActive) {
      setShowPendingIndicator(false);
      return;
    }

    const spinnerTimer = window.setTimeout(() => {
      setShowPendingIndicator(true);
    }, 150);

    return () => window.clearTimeout(spinnerTimer);
  }, [loading, isRetrievalActive]);

  const handleClear = () => {
    setLocalSearchTerm('');
    setSearchTerm('');
  };

  return (
    <div
      className={cn(
        'relative flex items-center border border-border/70 bg-background/78 shadow-[0_6px_18px_rgba(15,23,42,0.05)] backdrop-blur-xl',
        compact
          ? 'rounded-[12px] min-[1200px]:rounded-[12px]'
          : 'rounded-xl min-[1200px]:rounded-2xl',
        className
      )}
    >
      <Search
        className={cn(
          'pointer-events-none absolute h-4 w-4 text-muted-foreground',
          compact ? 'left-2.5' : 'left-3.5 min-[1200px]:left-4'
        )}
      />
      <Input
        type="search"
        inputMode="search"
        aria-label={t('common:search')}
        placeholder={t('clipboard:retrieval.searchPlaceholder', {
          defaultValue: '搜索内容、URL host、JSON key、命令或应用...',
        })}
        value={localSearchTerm}
        onChange={(e) => setLocalSearchTerm(e.target.value)}
        className={cn(
          'border-0 bg-transparent text-sm shadow-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-0',
          compact
            ? 'h-7 rounded-[9px] pl-8 pr-8 text-[12px]'
            : 'h-10 rounded-xl pl-10 pr-11 min-[1200px]:h-12 min-[1200px]:rounded-2xl min-[1200px]:pl-11 min-[1200px]:pr-12'
        )}
      />

      {showPendingIndicator && (
        <span
          aria-hidden="true"
          className={cn(
            'absolute inline-flex items-center justify-center text-primary',
            localSearchTerm ? 'right-9' : 'right-3',
            compact ? 'h-4 w-4' : 'h-4 w-4 min-[1200px]:right-4'
          )}
        >
          <span className="h-4 w-4 animate-spin rounded-full border-2 border-primary/60 border-t-transparent" />
        </span>
      )}

      {localSearchTerm && (
        <Button
          type="button"
          variant="ghost"
          size="icon"
          aria-label={t('common:cancel')}
          onClick={handleClear}
          className={cn(
            'absolute rounded-full text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground',
            compact
              ? 'right-1 h-[22px] w-[22px]'
              : 'right-1.5 h-7 w-7 min-[1200px]:right-2 min-[1200px]:h-8 min-[1200px]:w-8'
          )}
        >
          <X className="h-4 w-4" />
        </Button>
      )}
    </div>
  );
};
