import { Globe, Link2 } from 'lucide-react';
import { Badge } from '../../ui/badge';

export interface UrlCardRendererProps {
  raw: string;
  parts?: {
    protocol: string;
    host: string;
    path: string;
    query_params: Array<[string, string]>;
  } | null;
  preview?: {
    title?: string;
    description?: string;
    finalUrl?: string;
    status?: number;
    contentType?: string;
  } | null;
}

const parseUrlParts = (raw: string): UrlCardRendererProps['parts'] => {
  const trimmed = raw.trim();
  if (!trimmed) {
    return null;
  }

  const candidates =
    trimmed.startsWith('http://') || trimmed.startsWith('https://')
      ? [trimmed]
      : [trimmed, `https://${trimmed}`];

  for (const candidate of candidates) {
    try {
      const parsed = new URL(candidate);
      return {
        protocol: parsed.protocol.replace(/:$/, ''),
        host: parsed.host,
        path: parsed.pathname || '/',
        query_params: Array.from(parsed.searchParams.entries()),
      };
    } catch {
      // try next candidate
    }
  }

  return null;
};

const readOnlyFallback = 'Not available';

const normalizePreviewText = (value?: string | null) => {
  const normalized = value?.replace(/\s+/g, ' ').trim();
  return normalized || '';
};

const InfoRow = ({ label, value }: { label: string; value: string }) => (
  <div className="space-y-1 rounded-xl border border-border/60 bg-background/70 p-3">
    <div className="text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground">
      {label}
    </div>
    <div className="break-all font-mono text-sm text-foreground">{value}</div>
  </div>
);

export function UrlCardRenderer({ raw, parts, preview }: UrlCardRendererProps) {
  const resolvedParts = parts ?? parseUrlParts(raw);
  const queryParams = resolvedParts?.query_params ?? [];
  const pathValue = resolvedParts?.path || '/';
  const queryCountLabel = queryParams.length === 1 ? '1 param' : `${queryParams.length} params`;
  const pageTitle = normalizePreviewText(preview?.title);
  const pageDescription = normalizePreviewText(preview?.description);
  const responseStatus =
    typeof preview?.status === 'number' && Number.isFinite(preview.status)
      ? `HTTP ${preview.status}`
      : '';
  const contentType = normalizePreviewText(preview?.contentType);
  const redirectedUrl =
    preview?.finalUrl && preview.finalUrl !== raw ? normalizePreviewText(preview.finalUrl) : '';
  const summaryBadges = [responseStatus, contentType].filter(Boolean);

  return (
    <div className="space-y-4 rounded-[16px] border border-border/70 bg-background/70 p-4 shadow-none">
      <div className="flex flex-wrap items-center gap-2">
        <div className="flex h-9 w-9 items-center justify-center rounded-2xl border border-primary/15 bg-primary/10 text-primary">
          <Globe className="h-4 w-4" />
        </div>
        <Badge variant="secondary">URL</Badge>
        <Badge variant="outline">{queryCountLabel}</Badge>
      </div>

      {(pageTitle || pageDescription || summaryBadges.length > 0) && (
        <div className="space-y-3 rounded-2xl border border-border/60 bg-muted/20 p-3.5">
          {pageTitle ? (
            <div className="text-base font-semibold leading-6 text-foreground">{pageTitle}</div>
          ) : null}
          {pageDescription ? (
            <p className="text-sm leading-6 text-muted-foreground">{pageDescription}</p>
          ) : null}
          {summaryBadges.length > 0 ? (
            <div className="flex flex-wrap items-center gap-2">
              {summaryBadges.map((badge) => (
                <Badge key={badge} variant="outline" className="bg-background/80">
                  {badge}
                </Badge>
              ))}
            </div>
          ) : null}
        </div>
      )}

      <div className="grid gap-3 min-[900px]:grid-cols-2">
        <InfoRow label="Protocol" value={resolvedParts?.protocol || readOnlyFallback} />
        <InfoRow label="Host" value={resolvedParts?.host || readOnlyFallback} />
        <InfoRow label="Path" value={pathValue} />
        <InfoRow label="Params" value={queryCountLabel} />
        {redirectedUrl ? <InfoRow label="Resolved URL" value={redirectedUrl} /> : null}
      </div>

      <div className="space-y-2 rounded-xl border border-border/60 bg-muted/20 p-3">
        <div className="text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground">
          Query
        </div>
        {queryParams.length > 0 ? (
          <div className="space-y-2">
            {queryParams.map(([key, value], index) => (
              <div
                key={`${key}:${value}:${index}`}
                className="flex flex-wrap items-center gap-2 rounded-lg bg-background/80 px-3 py-2 text-sm"
              >
                <span className="font-mono text-foreground">{key}</span>
                <span className="text-muted-foreground">=</span>
                <span className="font-mono text-foreground">{value || '(empty)'}</span>
              </div>
            ))}
          </div>
        ) : (
          <div className="text-sm text-muted-foreground">No query parameters</div>
        )}
      </div>

      <div className="space-y-2 rounded-xl border border-border/60 bg-muted/20 p-3">
        <div className="flex items-center gap-2 text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground">
          <Link2 className="h-3.5 w-3.5" />
          <span>Raw URL</span>
        </div>
        <code className="block break-all rounded-lg bg-background/80 p-3 text-xs text-foreground">
          {raw}
        </code>
      </div>
    </div>
  );
}
