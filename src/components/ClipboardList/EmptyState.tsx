import React from 'react';
import { Clipboard } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { useClipboardStore } from '../../stores/clipboardStore';

export const EmptyState: React.FC = () => {
  const { t } = useTranslation('clipboard');
  const { isMonitoring, startMonitoring } = useClipboardStore();

  return (
    <Card className="flex h-full min-h-[280px] flex-col items-center justify-center rounded-[22px] border border-dashed border-border/80 bg-card/85 p-6 text-center shadow-[0_16px_50px_rgba(15,23,42,0.08)] backdrop-blur-xl min-[1200px]:min-h-[320px] min-[1200px]:rounded-[26px] min-[1200px]:p-8">
      <div className="mb-4 flex h-16 w-16 items-center justify-center rounded-[20px] border border-primary/15 bg-primary/10 text-primary min-[1200px]:mb-5 min-[1200px]:h-20 min-[1200px]:w-20 min-[1200px]:rounded-[24px]">
        <Clipboard size={34} />
      </div>
      <h3 className="mb-2 text-base font-semibold min-[1200px]:text-lg">
        {t('emptyState.noHistory')}
      </h3>
      {!isMonitoring ? (
        <>
          <p className="mb-4 max-w-xs text-sm text-muted-foreground min-[1200px]:mb-5">
            {t('emptyState.clickToStart')}
          </p>
          <Button
            onClick={startMonitoring}
            className="rounded-xl px-4 min-[1200px]:rounded-2xl min-[1200px]:px-5"
          >
            {t('actions.startMonitoring')}
          </Button>
        </>
      ) : (
        <p className="max-w-xs text-sm text-muted-foreground">{t('emptyState.copyToStart')}</p>
      )}
    </Card>
  );
};
