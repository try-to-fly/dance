import React from 'react';
import { AppWindow, Copy, FileImage, FileText, FolderClosed, Star, TrendingUp } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useClipboardStore } from '../../stores/clipboardStore';
import type { Statistics } from '../../types/clipboard';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '../ui/dialog';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import { cn } from '../../lib/utils';

interface StatisticsModalProps {
  isOpen: boolean;
  onClose: () => void;
  statistics?: Statistics | null;
}

export const StatisticsModal: React.FC<StatisticsModalProps> = ({
  isOpen,
  onClose,
  statistics: propStatistics,
}) => {
  const { t } = useTranslation(['statistics', 'common', 'clipboard']);
  const { statistics: storeStatistics, copyToClipboard } = useClipboardStore();

  const statistics = propStatistics || storeStatistics;

  if (!statistics) {
    return null;
  }

  const handleCopyContent = async (content: string | null) => {
    if (content) {
      await copyToClipboard(content);
    }
  };

  const getEntryIcon = (contentType: string) => {
    if (contentType.includes('image')) return FileImage;
    if (contentType.includes('file')) return FolderClosed;
    return FileText;
  };

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="max-w-4xl overflow-hidden border border-border/70 bg-background/95 p-0 shadow-[0_28px_90px_rgba(15,23,42,0.28)] backdrop-blur-2xl">
        <DialogHeader className="border-b border-border/70 px-6 py-5">
          <DialogTitle className="flex items-center gap-3 text-xl">
            <span className="flex h-11 w-11 items-center justify-center rounded-2xl border border-primary/20 bg-primary/10 text-primary">
              <TrendingUp className="h-5 w-5" />
            </span>
            {t('statistics:title')}
          </DialogTitle>
        </DialogHeader>

        <div className="max-h-[75vh] overflow-y-auto px-6 py-6">
          <div className="grid gap-4 md:grid-cols-2">
            <div className="rounded-[24px] border border-border/70 bg-card/80 p-5 shadow-[0_16px_40px_rgba(15,23,42,0.08)]">
              <div className="text-xs uppercase tracking-[0.28em] text-muted-foreground">
                {t('statistics:totalEntries')}
              </div>
              <div className="mt-4 text-4xl font-semibold tracking-tight text-foreground">
                {statistics.total_entries}
              </div>
            </div>

            <div className="rounded-[24px] border border-border/70 bg-card/80 p-5 shadow-[0_16px_40px_rgba(15,23,42,0.08)]">
              <div className="text-xs uppercase tracking-[0.28em] text-muted-foreground">
                {t('statistics:totalCopies')}
              </div>
              <div className="mt-4 text-4xl font-semibold tracking-tight text-foreground">
                {statistics.total_copies}
              </div>
            </div>
          </div>

          <div className="mt-6 grid gap-6 xl:grid-cols-[1.35fr_1fr]">
            <section className="rounded-[24px] border border-border/70 bg-card/80 p-5 shadow-[0_16px_40px_rgba(15,23,42,0.08)]">
              <div className="flex items-center justify-between gap-3">
                <h3 className="text-base font-semibold">{t('statistics:mostCopied')}</h3>
                <Badge
                  variant="secondary"
                  className="rounded-full border border-primary/20 bg-primary/10 px-3 py-1 text-primary"
                >
                  {statistics.most_copied.length}
                </Badge>
              </div>

              <div className="mt-4 space-y-3">
                {statistics.most_copied.length > 0 ? (
                  statistics.most_copied.slice(0, 5).map((entry) => {
                    const contentType = entry.content_type.toLowerCase();
                    const EntryIcon = getEntryIcon(contentType);

                    return (
                      <div
                        key={entry.id}
                        className="flex items-start gap-3 rounded-[20px] border border-border/70 bg-background/70 p-4"
                      >
                        <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl border border-border/70 bg-secondary/70 text-muted-foreground">
                          <EntryIcon className="h-5 w-5" />
                        </div>

                        <div className="min-w-0 flex-1">
                          <div className="flex items-center gap-2">
                            <Badge
                              variant="secondary"
                              className="rounded-full px-2.5 py-1 text-[11px]"
                            >
                              {contentType.includes('image')
                                ? t('clipboard:contentTypes.image')
                                : contentType.includes('file')
                                  ? t('clipboard:contentTypes.file')
                                  : t('detail.contentTypes.text')}
                            </Badge>
                            {entry.is_favorite && (
                              <Star
                                className="h-4 w-4 text-amber-500 dark:text-amber-300"
                                fill="currentColor"
                              />
                            )}
                          </div>

                          <div className="mt-3 line-clamp-2 break-words text-sm font-medium text-foreground">
                            {entry.content_data || entry.file_path || '(无内容)'}
                          </div>

                          <div className="mt-3 text-xs text-muted-foreground">
                            {t('statistics:copiedTimes', { count: entry.copy_count })}
                          </div>
                        </div>

                        <Button
                          type="button"
                          variant="outline"
                          size="icon"
                          aria-label={t('common:copy')}
                          onClick={() => handleCopyContent(entry.content_data)}
                          disabled={!entry.content_data}
                          className="h-9 w-9 rounded-full"
                        >
                          <Copy className="h-4 w-4" />
                        </Button>
                      </div>
                    );
                  })
                ) : (
                  <div className="rounded-[20px] border border-dashed border-border bg-background/60 px-4 py-10 text-center text-sm text-muted-foreground">
                    {t('common:noData')}
                  </div>
                )}
              </div>
            </section>

            <section className="rounded-[24px] border border-border/70 bg-card/80 p-5 shadow-[0_16px_40px_rgba(15,23,42,0.08)]">
              <div className="flex items-center justify-between gap-3">
                <h3 className="text-base font-semibold">{t('statistics:appUsage')}</h3>
                <span className="flex h-10 w-10 items-center justify-center rounded-2xl border border-primary/20 bg-primary/10 text-primary">
                  <AppWindow className="h-4 w-4" />
                </span>
              </div>

              <div className="mt-4 space-y-3">
                {statistics.recent_apps.length > 0 ? (
                  statistics.recent_apps.slice(0, 8).map((app, index) => (
                    <div
                      key={`${app.app_name}-${index}`}
                      className={cn(
                        'flex items-center justify-between gap-3 rounded-[20px] border border-border/70 bg-background/70 px-4 py-3',
                        index === 0 && 'border-primary/20 bg-primary/5'
                      )}
                    >
                      <div className="min-w-0">
                        <div className="truncate text-sm font-medium text-foreground">
                          {app.app_name || t('common:unknownApp')}
                        </div>
                      </div>
                      <Badge variant="outline" className="rounded-full border-border/70 px-3 py-1">
                        {t('statistics:times', { count: app.count })}
                      </Badge>
                    </div>
                  ))
                ) : (
                  <div className="rounded-[20px] border border-dashed border-border bg-background/60 px-4 py-10 text-center text-sm text-muted-foreground">
                    {t('common:noData')}
                  </div>
                )}
              </div>
            </section>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};
