import { useEffect, useState } from 'react';
import { PreviewAlternateView } from '../../../types/clipboard';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../../ui/tabs';
import { Card, CardContent } from '../../ui/card';
import { JsonRenderer, UnifiedTextRenderer } from '../ContentRenderers';

interface AlternateViewsProps {
  views: PreviewAlternateView[];
}

const normalizeToText = (payload: unknown): string => {
  if (typeof payload === 'string') {
    return payload;
  }

  try {
    return JSON.stringify(payload, null, 2);
  } catch {
    return String(payload);
  }
};

const normalizeToMediaSrc = (payload: unknown) => {
  if (typeof payload === 'string') {
    return payload;
  }

  return normalizeToText(payload);
};

export function AlternateViews({ views }: AlternateViewsProps) {
  const defaultKey = views[0]?.key ?? '';
  const [activeKey, setActiveKey] = useState(defaultKey);

  useEffect(() => {
    if (!views.some((view) => view.key === activeKey)) {
      setActiveKey(defaultKey);
    }
  }, [activeKey, defaultKey, views]);

  if (views.length === 0) {
    return null;
  }

  if (views.length === 1 && views[0]?.key === 'raw') {
    return null;
  }

  const renderView = (view: PreviewAlternateView) => {
    if (view.kind === 'json') {
      return <JsonRenderer content={normalizeToText(view.payload)} />;
    }

    if (view.kind === 'image') {
      return (
        <div className="overflow-hidden rounded-xl border border-border/70 bg-muted/30 p-2">
          <img
            src={normalizeToMediaSrc(view.payload)}
            alt={`${view.label} preview`}
            className="max-h-[320px] w-full rounded-lg object-contain"
          />
        </div>
      );
    }

    if (view.kind === 'audio') {
      return (
        <audio
          controls
          aria-label={`${view.label} preview`}
          className="w-full"
          src={normalizeToMediaSrc(view.payload)}
        />
      );
    }

    if (view.kind === 'video') {
      return (
        <video
          controls
          aria-label={`${view.label} preview`}
          className="max-h-[320px] w-full rounded-xl border border-border/70 bg-muted/30"
          src={normalizeToMediaSrc(view.payload)}
        />
      );
    }

    if (view.kind === 'url_card') {
      return (
        <code className="block break-all rounded-xl bg-muted/70 p-2.5 text-[11px] leading-5">
          {normalizeToText(view.payload)}
        </code>
      );
    }

    return (
      <UnifiedTextRenderer
        content={normalizeToText(view.payload)}
        contentSubType={
          view.kind === 'markdown' ? 'markdown' : view.kind === 'code' ? 'code' : 'plain_text'
        }
      />
    );
  };

  if (views.length === 1) {
    return (
      <Card className="rounded-[16px] border-border/70 bg-background/70 shadow-none">
        <CardContent className="space-y-2 p-2.5 min-[1200px]:p-3">
          <div className="px-1 text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground">
            {views[0].label}
          </div>
          {renderView(views[0])}
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="rounded-[16px] border-border/70 bg-background/70 shadow-none">
      <CardContent className="p-2.5 min-[1200px]:p-3">
        <Tabs value={activeKey} onValueChange={setActiveKey}>
          <TabsList className="mb-2.5 h-8 w-full justify-start gap-1 overflow-x-auto rounded-full bg-muted/60 p-1">
            {views.map((view) => (
              <TabsTrigger
                key={view.key}
                value={view.key}
                className="rounded-full px-2.5 py-1 text-[11px]"
              >
                {view.label}
              </TabsTrigger>
            ))}
          </TabsList>

          {views.map((view) => (
            <TabsContent key={view.key} value={view.key} className="mt-0">
              {renderView(view)}
            </TabsContent>
          ))}
        </Tabs>
      </CardContent>
    </Card>
  );
}
