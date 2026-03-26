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
  const { searchTerm, setSearchTerm, selectedType } = useClipboardStore();
  const [localSearchTerm, setLocalSearchTerm] = useState(searchTerm);
  const deferredSearchTerm = useDeferredValue(localSearchTerm);

  useEffect(() => {
    setLocalSearchTerm(searchTerm);
  }, [searchTerm]);

  useEffect(() => {
    const debounceTimer = window.setTimeout(() => {
      setSearchTerm(deferredSearchTerm);

      if (deferredSearchTerm.trim()) {
        analytics.track(ANALYTICS_EVENTS.SEARCH_PERFORMED, {
          has_filter: selectedType !== 'all' ? 1 : 0,
          filter_type: selectedType,
        });
      }
    }, 200);

    return () => window.clearTimeout(debounceTimer);
  }, [deferredSearchTerm, selectedType, setSearchTerm]);

  const handleClear = () => {
    setLocalSearchTerm('');
    setSearchTerm('');
  };

  return (
    <div
      className={cn(
        'relative flex items-center border border-border/70 bg-background/75 shadow-[0_10px_30px_rgba(15,23,42,0.08)] backdrop-blur-xl',
        compact ? 'rounded-xl min-[1200px]:rounded-xl' : 'rounded-xl min-[1200px]:rounded-2xl',
        className
      )}
    >
      <Search
        className={cn(
          'pointer-events-none absolute h-4 w-4 text-muted-foreground',
          compact ? 'left-3' : 'left-3.5 min-[1200px]:left-4'
        )}
      />
      <Input
        type="search"
        inputMode="search"
        aria-label={t('common:search')}
        placeholder={t('common:search')}
        value={localSearchTerm}
        onChange={(e) => setLocalSearchTerm(e.target.value)}
        className={cn(
          'border-0 bg-transparent text-sm shadow-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-0',
          compact
            ? 'h-9 rounded-xl pl-9 pr-10'
            : 'h-10 rounded-xl pl-10 pr-11 min-[1200px]:h-12 min-[1200px]:rounded-2xl min-[1200px]:pl-11 min-[1200px]:pr-12'
        )}
      />

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
              ? 'right-1 h-7 w-7'
              : 'right-1.5 h-7 w-7 min-[1200px]:right-2 min-[1200px]:h-8 min-[1200px]:w-8'
          )}
        >
          <X className="h-4 w-4" />
        </Button>
      )}
    </div>
  );
};
