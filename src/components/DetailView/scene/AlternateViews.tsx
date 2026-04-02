import { useEffect, useState } from 'react';
import { PreviewAlternateView } from '../../../types/clipboard';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../../ui/tabs';
import { Card, CardContent } from '../../ui/card';
import { JsonRenderer, UnifiedTextRenderer } from '../ContentRenderers';
import { UrlCardRenderer, type UrlCardRendererProps } from '../ContentRenderers/UrlCardRenderer';

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

const normalizeUrlCardPayload = (payload: unknown): UrlCardRendererProps => {
  if (payload && typeof payload === 'object') {
    const candidate = payload as Partial<UrlCardRendererProps>;
    if (typeof candidate.raw === 'string') {
      return {
        raw: candidate.raw,
        parts: candidate.parts ?? null,
      };
    }
  }

  return {
    raw: normalizeToText(payload),
    parts: null,
  };
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
      return <UrlCardRenderer {...normalizeUrlCardPayload(view.payload)} />;
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

  return (
    <Card className="overflow-hidden rounded-[16px] border-border/70 bg-card/95 shadow-[0_8px_20px_rgba(15,23,42,0.04)]">
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
            <TabsContent key={view.key} value={view.key} forceMount className="mt-0">
              {renderView(view)}
            </TabsContent>
          ))}
        </Tabs>
      </CardContent>
    </Card>
  );
}
