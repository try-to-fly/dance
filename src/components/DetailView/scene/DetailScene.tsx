import {
  AppWindow,
  ClipboardPaste,
  Copy,
  ExternalLink,
  FileCode2,
  FileImage,
  FileText,
  FolderClosed,
  FolderOpen,
  Heart,
  Trash2,
} from 'lucide-react';
import { ComponentType } from 'react';
import {
  ClipboardEntry,
  PreviewAction,
  PreviewDescriptor,
  PreviewKind,
} from '../../../types/clipboard';
import { Badge } from '../../ui/badge';
import { Button } from '../../ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '../../ui/card';
import { cn } from '../../../lib/utils';
import { AlternateViews } from './AlternateViews';
import { InspectorPanel } from './InspectorPanel';
import { PrimaryPreviewRenderer } from './PrimaryPreviewRenderer';

const immersivePreviewKinds = new Set<PreviewKind>(['image', 'video', 'audio']);
const denseHeaderPreviewKinds = new Set<PreviewKind>([
  'image',
  'video',
  'audio',
  'json',
  'code',
  'markdown',
  'base64_text',
  'base64_binary',
]);

interface DetailSceneProps {
  entry: ClipboardEntry;
  descriptor: PreviewDescriptor;
  metadataPills: Array<{
    key: string;
    icon: ComponentType<{ className?: string }>;
    label: string;
    value: string;
    fullValue: string;
  }>;
  labels: {
    copy: string;
    copyDecoded: string;
    paste: string;
    delete: string;
    favorite: string;
    unfavorite: string;
    openFile: string;
    openUrl: string;
    title: string;
  };
  onCopy: () => Promise<void> | void;
  onCopyDecoded: () => Promise<void> | void;
  onPaste: () => Promise<void> | void;
  onDelete: () => Promise<void> | void;
  onToggleFavorite: () => Promise<void> | void;
  onOpenUrl: () => void;
  onOpenFile: () => Promise<void> | void;
  canCopyDecoded?: boolean;
}

export function DetailScene({
  entry,
  descriptor,
  metadataPills,
  labels,
  onCopy,
  onCopyDecoded,
  onPaste,
  onDelete,
  onToggleFavorite,
  onOpenUrl,
  onOpenFile,
  canCopyDecoded = false,
}: DetailSceneProps) {
  const contentType = entry.content_type.toLowerCase();
  const contentIsImage = contentType.includes('image');
  const contentIsFile = contentType.includes('file');
  const hasInspector = descriptor.inspectorSections.length > 0;
  const hasImmersivePreview = immersivePreviewKinds.has(descriptor.primaryKind);
  const useDenseHeader = denseHeaderPreviewKinds.has(descriptor.primaryKind);
  const hasRawOnlyAlternateView =
    descriptor.alternateViews.length === 1 && descriptor.alternateViews[0]?.key === 'raw';
  const showAlternateViews =
    descriptor.alternateViews.length > 0 && !(hasImmersivePreview && hasRawOnlyAlternateView);
  const layoutMode = hasImmersivePreview ? 'immersive' : useDenseHeader ? 'dense' : 'default';
  const actionButtonClassName = cn(
    'h-8 rounded-xl px-3 text-xs min-[1200px]:h-9',
    useDenseHeader ? 'min-[1200px]:rounded-xl' : 'min-[1200px]:rounded-2xl'
  );
  const iconButtonClassName = cn(
    'h-8 w-8 rounded-xl min-[1200px]:h-9 min-[1200px]:w-9',
    useDenseHeader ? 'min-[1200px]:rounded-xl' : 'min-[1200px]:rounded-2xl'
  );

  const renderActionButton = (action: PreviewAction) => {
    switch (action) {
      case 'copy_raw':
        return (
          <Button
            key={action}
            type="button"
            variant="secondary"
            size="sm"
            onClick={onCopy}
            disabled={!entry.content_data}
            aria-label={labels.copy}
            title={labels.copy}
            className={actionButtonClassName}
          >
            <Copy className="mr-1.5 h-4 w-4" />
            {labels.copy}
          </Button>
        );
      case 'copy_decoded':
        return (
          <Button
            key={action}
            type="button"
            variant="outline"
            size="sm"
            onClick={onCopyDecoded}
            disabled={!canCopyDecoded}
            aria-label={labels.copyDecoded}
            title={labels.copyDecoded}
            className={actionButtonClassName}
          >
            <FileCode2 className="mr-1.5 h-4 w-4" />
            {labels.copyDecoded}
          </Button>
        );
      case 'paste':
        return (
          <Button
            key={action}
            type="button"
            variant="outline"
            size="sm"
            onClick={onPaste}
            aria-label={labels.paste}
            title={labels.paste}
            className={actionButtonClassName}
          >
            <ClipboardPaste className="mr-1.5 h-4 w-4" />
            {labels.paste}
          </Button>
        );
      case 'open_url':
        return (
          <Button
            key={action}
            type="button"
            variant="outline"
            size="sm"
            onClick={onOpenUrl}
            aria-label={labels.openUrl}
            title={labels.openUrl}
            className={actionButtonClassName}
          >
            <ExternalLink className="mr-1.5 h-4 w-4" />
            {labels.openUrl}
          </Button>
        );
      case 'open_file':
        return (
          <Button
            key={action}
            type="button"
            variant="outline"
            size="sm"
            onClick={onOpenFile}
            aria-label={labels.openFile}
            title={labels.openFile}
            className={actionButtonClassName}
          >
            <FolderOpen className="mr-1.5 h-4 w-4" />
            {labels.openFile}
          </Button>
        );
      default:
        return null;
    }
  };

  return (
    <Card
      id="detail-view"
      data-layout={layoutMode}
      className="flex h-full min-h-0 flex-col overflow-hidden rounded-[20px] border border-border/70 bg-card/88 shadow-[0_16px_44px_rgba(15,23,42,0.08)] backdrop-blur-xl min-[1200px]:rounded-[24px]"
    >
      <CardHeader
        id="detail-view-header"
        className={cn(
          'space-y-0 border-b border-border/70',
          useDenseHeader
            ? 'gap-1.5 px-2.5 pb-2 pt-2.5 min-[1200px]:gap-2 min-[1200px]:px-3 min-[1200px]:pb-2.5 min-[1200px]:pt-3'
            : 'gap-2.5 px-3.5 pb-2.5 pt-3.5 min-[1200px]:gap-3 min-[1200px]:px-4 min-[1200px]:pb-3 min-[1200px]:pt-4'
        )}
      >
        <div
          className={cn(
            'flex flex-wrap items-start justify-between',
            useDenseHeader ? 'gap-2.5 min-[1200px]:gap-3' : 'gap-3 min-[1200px]:gap-4'
          )}
        >
          <div className={cn('min-w-0 flex-1', useDenseHeader ? 'space-y-1.5' : 'space-y-2')}>
            <div className="flex flex-wrap items-center gap-2">
              <Badge
                id="detail-view-type-badge"
                variant="secondary"
                className={cn(
                  'rounded-full border border-primary/20 bg-primary/10 text-primary',
                  useDenseHeader
                    ? 'px-2 py-0.5 text-[10px] min-[1200px]:px-2.5 min-[1200px]:text-[11px]'
                    : 'px-2.5 py-1 text-[11px] min-[1200px]:px-3'
                )}
              >
                {contentIsImage ? (
                  <FileImage className="mr-2 h-3.5 w-3.5" />
                ) : contentIsFile ? (
                  <FolderClosed className="mr-2 h-3.5 w-3.5" />
                ) : (
                  <FileText className="mr-2 h-3.5 w-3.5" />
                )}
                {descriptor.typeLabel}
              </Badge>

              {descriptor.badges.map((badge) => (
                <Badge
                  key={badge.label}
                  variant={badge.tone === 'warning' ? 'outline' : 'secondary'}
                  className={cn(
                    'rounded-full',
                    badge.tone === 'warning' &&
                      'border-amber-200 bg-amber-50 text-amber-700 dark:border-amber-400/20 dark:bg-amber-400/10 dark:text-amber-300',
                    useDenseHeader
                      ? 'px-2 py-0.5 text-[10px] min-[1200px]:px-2.5 min-[1200px]:text-[11px]'
                      : 'px-2.5 py-1 text-[11px] min-[1200px]:px-3'
                  )}
                >
                  {badge.label}
                </Badge>
              ))}

              {entry.is_favorite && (
                <Badge
                  variant="outline"
                  className={cn(
                    'rounded-full border-amber-200 bg-amber-50 text-amber-700 dark:border-amber-400/20 dark:bg-amber-400/10 dark:text-amber-300',
                    useDenseHeader
                      ? 'px-2 py-0.5 text-[10px] min-[1200px]:px-2.5 min-[1200px]:text-[11px]'
                      : 'px-2.5 py-1 text-[11px] min-[1200px]:px-3'
                  )}
                >
                  <Heart className="mr-2 h-3.5 w-3.5" fill="currentColor" />
                  {labels.favorite}
                </Badge>
              )}
            </div>

            <div className={cn(useDenseHeader ? 'space-y-0.5' : 'space-y-1')}>
              {!useDenseHeader && (
                <p className="text-[10px] font-semibold uppercase tracking-[0.26em] text-muted-foreground min-[1200px]:text-[11px]">
                  {labels.title}
                </p>
              )}
              <CardTitle
                id="detail-view-title"
                className={cn(
                  'tracking-tight',
                  useDenseHeader
                    ? 'line-clamp-2 text-[15px] leading-tight min-[1200px]:line-clamp-1 min-[1200px]:text-base'
                    : 'line-clamp-2 text-[15px] leading-snug min-[1200px]:text-[17px]'
                )}
                title={descriptor.headline}
              >
                {descriptor.headline}
              </CardTitle>
            </div>
          </div>

          <div className="flex shrink-0 flex-wrap items-center justify-end gap-1.5">
            {descriptor.actions.map(renderActionButton)}
            <Button
              type="button"
              variant="outline"
              size="icon"
              onClick={onToggleFavorite}
              aria-label={entry.is_favorite ? labels.unfavorite : labels.favorite}
              title={entry.is_favorite ? labels.unfavorite : labels.favorite}
              className={cn(
                iconButtonClassName,
                entry.is_favorite && 'border-primary/30 bg-primary/10 text-primary'
              )}
            >
              <Heart className="h-4 w-4" fill={entry.is_favorite ? 'currentColor' : 'none'} />
            </Button>
            <Button
              type="button"
              variant="outline"
              size="icon"
              onClick={onDelete}
              aria-label={labels.delete}
              title={labels.delete}
              className={cn(iconButtonClassName, 'text-destructive hover:text-destructive')}
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>
        </div>

        <div
          id="detail-view-metadata"
          className={cn(
            'flex flex-wrap items-center text-muted-foreground',
            useDenseHeader ? 'gap-x-2 gap-y-1 text-[11px]' : 'gap-x-2.5 gap-y-1 text-[11px]'
          )}
        >
          {metadataPills.map((item) => {
            const Icon = item.icon;

            return (
              <div
                key={item.key}
                title={`${item.label}: ${item.fullValue}`}
                className={cn(
                  'inline-flex min-w-0 max-w-full items-center rounded-full border border-border/60 bg-background/70',
                  useDenseHeader ? 'gap-1 px-1.5 py-0.5' : 'gap-1.5 px-2 py-1'
                )}
              >
                <Icon className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                {!useDenseHeader && <span className="shrink-0 text-[10px]">{item.label}</span>}
                <span className="min-w-0 truncate font-medium text-foreground/90">
                  {item.value}
                </span>
              </div>
            );
          })}
        </div>
      </CardHeader>

      <CardContent
        id="detail-view-content"
        className={cn(
          'flex min-h-0 flex-1 flex-col',
          hasImmersivePreview ? 'p-1.5 min-[1200px]:p-2' : 'p-2 min-[1200px]:p-2.5'
        )}
      >
        <div
          id="detail-view-shell"
          className={cn(
            'flex h-full min-h-0 flex-1 flex-col overflow-hidden border border-border/70 bg-background/70',
            hasImmersivePreview
              ? 'rounded-[15px] min-[1200px]:rounded-[18px]'
              : 'rounded-[16px] min-[1200px]:rounded-[18px]'
          )}
        >
          <div
            id="detail-view-content-wrapper"
            className={cn(
              'grid min-h-0 flex-1',
              hasImmersivePreview
                ? 'gap-1.5 p-1.5 min-[1200px]:gap-2 min-[1200px]:p-2'
                : 'gap-2 p-2 min-[1200px]:gap-2.5 min-[1200px]:p-2.5',
              hasInspector && 'min-[1200px]:grid-cols-[minmax(0,1fr)_280px]'
            )}
          >
            <div
              id="detail-view-primary-column"
              className={cn(
                'min-h-0',
                hasImmersivePreview ? 'flex flex-col gap-2' : 'overflow-y-auto pr-0.5'
              )}
            >
              <div
                className={cn(
                  'min-h-0',
                  hasImmersivePreview ? 'flex min-h-0 flex-1 flex-col gap-2' : 'space-y-2.5'
                )}
              >
                <div className={cn('min-h-0', hasImmersivePreview && 'flex-1')}>
                  <PrimaryPreviewRenderer
                    kind={descriptor.primaryKind}
                    payload={descriptor.primaryPayload}
                    onOpenFile={onOpenFile}
                  />
                </div>
                {showAlternateViews && <AlternateViews views={descriptor.alternateViews} />}
              </div>
            </div>

            {hasInspector && (
              <div className="min-h-0 overflow-y-auto pr-0.5">
                <InspectorPanel sections={descriptor.inspectorSections} />
              </div>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

export function DetailEmptyState({ selectItemLabel }: { selectItemLabel: string }) {
  return (
    <Card
      id="detail-view-empty"
      className="flex h-full min-h-[280px] flex-col rounded-[22px] border border-border/70 bg-card/88 shadow-[0_16px_50px_rgba(15,23,42,0.08)] backdrop-blur-xl min-[1200px]:min-h-[320px] min-[1200px]:rounded-[26px]"
    >
      <CardContent
        id="detail-view-empty-content"
        className="flex flex-1 items-center justify-center p-6 min-[1200px]:p-8"
      >
        <div id="detail-view-empty-message" className="max-w-sm text-center">
          <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-[20px] border border-primary/15 bg-primary/10 text-primary min-[1200px]:mb-5 min-[1200px]:h-20 min-[1200px]:w-20 min-[1200px]:rounded-[26px]">
            <AppWindow className="h-9 w-9" />
          </div>
          <p className="text-base font-medium text-foreground">{selectItemLabel}</p>
          <p className="mt-2 text-sm text-muted-foreground">Alt + 1-9</p>
        </div>
      </CardContent>
    </Card>
  );
}
