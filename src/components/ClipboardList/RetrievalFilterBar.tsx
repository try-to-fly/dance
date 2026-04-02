import { AppWindow, Star } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useClipboardStore } from '../../stores/clipboardStore';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select';
import { Button } from '../ui/button';
import { cn } from '../../lib/utils';

export function RetrievalFilterBar() {
  const { t } = useTranslation(['clipboard']);
  const {
    selectedSourceApp,
    setSelectedSourceApp,
    sourceAppOptions,
    favoritesOnly,
    setFavoritesOnly,
  } = useClipboardStore();

  return (
    <div className="border-b border-border/70 bg-background/55 px-2 py-2 backdrop-blur-xl min-[1200px]:px-2.5">
      <div className="flex flex-wrap items-center gap-1">
        <Select value={selectedSourceApp} onValueChange={setSelectedSourceApp}>
          <SelectTrigger className="h-[30px] min-w-[124px] max-w-[148px] rounded-[11px] border-border/70 bg-background/80 px-2 text-[12px] shadow-sm">
            <div className="flex min-w-0 items-center gap-2">
              <AppWindow className="h-3.5 w-3.5 text-muted-foreground" />
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
            'h-[30px] rounded-[11px] border-border/70 px-2 text-[12px] shadow-sm',
            favoritesOnly
              ? 'border-primary/20 bg-primary/10 text-primary hover:bg-primary/15'
              : 'bg-background/80 text-muted-foreground hover:bg-accent/70 hover:text-foreground'
          )}
        >
          <Star className="mr-1.5 h-3.5 w-3.5" fill={favoritesOnly ? 'currentColor' : 'none'} />
          {t('retrieval.favoritesOnly', { defaultValue: '收藏' })}
        </Button>
      </div>
    </div>
  );
}
