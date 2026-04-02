import React from 'react';
import { Clipboard, Search } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import { useClipboardStore } from '../../stores/clipboardStore';

interface EmptyStateProps {
  mode?: 'empty-history' | 'no-results';
  onResetFilters?: () => void;
}

export const EmptyState: React.FC<EmptyStateProps> = ({
  mode = 'empty-history',
  onResetFilters,
}) => {
  const { t } = useTranslation('clipboard');
  const { isMonitoring, startMonitoring } = useClipboardStore();
  const isNoResults = mode === 'no-results';
  const supportedTypes = [
    t('contentTypes.url'),
    t('contentTypes.json'),
    t('contentTypes.command'),
    t('contentTypes.image'),
  ];

  return (
    <Card className="flex h-full min-h-[220px] flex-col rounded-[18px] border border-dashed border-border/80 bg-card/85 p-3 shadow-[0_16px_50px_rgba(15,23,42,0.08)] backdrop-blur-xl min-[1200px]:min-h-[240px] min-[1200px]:rounded-[20px] min-[1200px]:p-3.5">
      <div className="flex flex-1 flex-col gap-2.5">
        <div className="rounded-[16px] border border-border/70 bg-background/65 p-3 min-[1200px]:p-3.5">
          <div className="mb-3 flex h-11 w-11 items-center justify-center rounded-[14px] border border-primary/15 bg-primary/10 text-primary min-[1200px]:h-12 min-[1200px]:w-12 min-[1200px]:rounded-[15px]">
            {isNoResults ? <Search size={22} /> : <Clipboard size={22} />}
          </div>
          <h3 className="mb-1.5 text-[14px] font-semibold min-[1200px]:text-[15px]">
            {isNoResults
              ? t('retrieval.noResultsTitle', { defaultValue: '未找到匹配结果' })
              : t('emptyState.noHistory')}
          </h3>
          {isNoResults ? (
            <>
              <p className="mb-3 max-w-[17rem] text-[12px] leading-[1.45] text-muted-foreground">
                {t('retrieval.noResultsBody', {
                  defaultValue: '尝试缩短关键词、切换类型，或清除来源与收藏筛选。',
                })}
              </p>
              <Button onClick={onResetFilters} className="h-[30px] rounded-[11px] px-3 text-[12px]">
                {t('retrieval.clearFilters', { defaultValue: '清除筛选' })}
              </Button>
            </>
          ) : !isMonitoring ? (
            <>
              <p className="mb-3 max-w-[17rem] text-[12px] leading-[1.45] text-muted-foreground">
                {t('emptyState.clickToStart')}
              </p>
              <Button
                onClick={startMonitoring}
                className="h-[30px] rounded-[11px] px-3 text-[12px]"
              >
                {t('actions.startMonitoring')}
              </Button>
            </>
          ) : (
            <p className="max-w-[17rem] text-[12px] leading-[1.45] text-muted-foreground">
              {t('emptyState.copyToStart')}
            </p>
          )}
        </div>

        {!isNoResults && (
          <div className="rounded-[14px] border border-border/70 bg-background/55 px-3 py-2.5">
            <div className="flex flex-wrap gap-1">
              {supportedTypes.map((label) => (
                <Badge
                  key={label}
                  variant="secondary"
                  className="rounded-full border border-border/60 bg-background/80 px-1.5 py-0.5 text-[10px] font-normal text-muted-foreground"
                >
                  {label}
                </Badge>
              ))}
            </div>
          </div>
        )}
      </div>
    </Card>
  );
};
