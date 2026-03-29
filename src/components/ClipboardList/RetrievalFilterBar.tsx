import { AppWindow, Clock3, Star } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useClipboardStore } from '../../stores/clipboardStore';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select';
import { Button } from '../ui/button';
import { cn } from '../../lib/utils';

const RECENCY_OPTIONS: Array<{ label: string; value: number | null }> = [
  { label: '全部', value: null },
  { label: '24 小时', value: 1 },
  { label: '7 天', value: 7 },
  { label: '30 天', value: 30 },
];

export function RetrievalFilterBar() {
  const { t } = useTranslation(['clipboard']);
  const {
    selectedSourceApp,
    setSelectedSourceApp,
    sourceAppOptions,
    favoritesOnly,
    setFavoritesOnly,
    recencyDays,
    setRecencyDays,
  } = useClipboardStore();

  return (
    <div className="border-b border-border/70 bg-background/55 px-3 py-3 backdrop-blur-xl min-[1200px]:px-3.5">
      <div className="flex flex-wrap items-center gap-2">
        <Select value={selectedSourceApp} onValueChange={setSelectedSourceApp}>
          <SelectTrigger className="h-9 min-w-[140px] max-w-[160px] rounded-xl border-border/70 bg-background/80 px-3 text-sm shadow-sm">
            <div className="flex min-w-0 items-center gap-2">
              <AppWindow className="h-4 w-4 text-muted-foreground" />
              <SelectValue
                placeholder={t('retrieval.sourceAppAll', {
                  defaultValue: '全部应用',
                })}
              />
            </div>
          </SelectTrigger>

          <SelectContent className="rounded-xl border-border/70 bg-popover/95 shadow-[0_16px_40px_rgba(15,23,42,0.16)] backdrop-blur-xl">
            <SelectItem value="all">
              {t('retrieval.sourceAppAll', { defaultValue: '全部应用' })}
            </SelectItem>
            {sourceAppOptions.map((sourceApp) => (
              <SelectItem key={sourceApp} value={sourceApp}>
                {sourceApp}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

        <Button
          type="button"
          variant="outline"
          onClick={() => setFavoritesOnly(!favoritesOnly)}
          className={cn(
            'h-9 rounded-xl border-border/70 px-3 text-sm shadow-sm',
            favoritesOnly
              ? 'border-primary/20 bg-primary/10 text-primary hover:bg-primary/15'
              : 'bg-background/80 text-muted-foreground hover:bg-accent/70 hover:text-foreground'
          )}
        >
          <Star className="mr-2 h-4 w-4" fill={favoritesOnly ? 'currentColor' : 'none'} />
          {t('retrieval.favoritesOnly', { defaultValue: '收藏' })}
        </Button>

        <div className="ml-auto flex flex-wrap items-center gap-1.5">
          {RECENCY_OPTIONS.map((option) => {
            const isActive = recencyDays === option.value;

            return (
              <Button
                key={option.label}
                type="button"
                variant="ghost"
                onClick={() => setRecencyDays(option.value)}
                className={cn(
                  'h-9 rounded-xl border px-3 text-sm shadow-sm transition-colors',
                  isActive
                    ? 'border-primary/20 bg-primary/10 text-primary hover:bg-primary/15'
                    : 'border-border/70 bg-background/80 text-muted-foreground hover:bg-accent/70 hover:text-foreground'
                )}
              >
                <Clock3 className="mr-2 h-4 w-4" />
                {option.label}
              </Button>
            );
          })}
        </div>
      </div>
    </div>
  );
}
