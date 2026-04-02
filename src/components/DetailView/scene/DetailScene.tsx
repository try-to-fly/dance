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
  Languages,
  MessageSquareText,
  MoreHorizontal,
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
import { Card, CardContent, CardHeader } from '../../ui/card';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '../../ui/dropdown-menu';
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
    aiTools: string;
    moreActions: string;
    translate?: string;
    chat?: string;
    title: string;
  };
  onCopy: () => Promise<void> | void;
  onCopyDecoded: () => Promise<void> | void;
  onPaste: () => Promise<void> | void;
  onDelete: () => Promise<void> | void;
  onToggleFavorite: () => Promise<void> | void;
  onOpenUrl: () => void;
  onOpenFile: () => Promise<void> | void;
  onTranslate?: () => Promise<void> | void;
  onOpenChat?: () => Promise<void> | void;
  canCopyDecoded?: boolean;
  showAiActions?: boolean;
}

export function DetailScene({
  entry,
  descriptor,
  metadataPills,
  labels,
  onCopy,
  onCopyDecoded,
  onPaste,
  onToggleFavorite,
  onOpenUrl,
  onOpenFile,
  onTranslate,
  onOpenChat,
  canCopyDecoded = false,
  showAiActions = false,
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
  const visibleActions = descriptor.actions;
  const hasAiButtons = showAiActions && Boolean(onTranslate || onOpenChat);
  const iconButtonClassName = cn(
    'h-[30px] w-[30px] rounded-[11px]',
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
            size="icon"
            onClick={onCopy}
            disabled={!entry.content_data}
            aria-label={labels.copy}
            title={labels.copy}
            className={iconButtonClassName}
          >
            <Copy className="h-4 w-4" />
          </Button>
        );
      case 'copy_decoded':
        return (
          <Button
            key={action}
            type="button"
            variant="outline"
            size="icon"
            onClick={onCopyDecoded}
            disabled={!canCopyDecoded}
            aria-label={labels.copyDecoded}
            title={labels.copyDecoded}
            className={iconButtonClassName}
          >
            <FileCode2 className="h-4 w-4" />
          </Button>
        );
      case 'paste':
        return (
          <Button
            key={action}
            type="button"
            variant="outline"
            size="icon"
            onClick={onPaste}
            aria-label={labels.paste}
            title={labels.paste}
            className={iconButtonClassName}
          >
            <ClipboardPaste className="h-4 w-4" />
          </Button>
        );
      case 'open_url':
        return (
          <Button
            key={action}
            type="button"
            variant="outline"
            size="icon"
            onClick={onOpenUrl}
            aria-label={labels.openUrl}
            title={labels.openUrl}
            className={iconButtonClassName}
          >
            <ExternalLink className="h-4 w-4" />
          </Button>
        );
      case 'open_file':
        return (
          <Button
            key={action}
            type="button"
            variant="outline"
            size="icon"
            onClick={onOpenFile}
            aria-label={labels.openFile}
            title={labels.openFile}
            className={iconButtonClassName}
          >
            <FolderOpen className="h-4 w-4" />
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
      className="isolate flex h-full min-h-0 flex-col overflow-hidden rounded-[18px] border border-border/70 bg-card/92 shadow-[0_16px_44px_rgba(15,23,42,0.08)] backdrop-blur-xl min-[1200px]:rounded-[20px]"
    >
      <CardHeader
        id="detail-view-header"
        className={cn(
          'space-y-0 border-b border-border/70',
          useDenseHeader
            ? 'gap-1 px-2 pb-1.5 pt-1.5 min-[1200px]:gap-1.5 min-[1200px]:px-2.5 min-[1200px]:pb-1.5 min-[1200px]:pt-2'
            : 'gap-1.5 px-2.5 pb-1.5 pt-2 min-[1200px]:gap-2 min-[1200px]:px-3 min-[1200px]:pb-2 min-[1200px]:pt-2.5'
        )}
      >
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

        <div
          className={cn(
            'flex flex-wrap items-center justify-between border-t border-border/60',
            useDenseHeader ? 'gap-2 pt-1.5 min-[1200px]:pt-2' : 'gap-2.5 pt-2'
          )}
        >
          <div
            id="detail-view-metadata"
            className={cn(
              'flex min-h-[38px] min-w-0 flex-1 flex-wrap content-center items-center text-muted-foreground',
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
                    useDenseHeader ? 'gap-1 px-1.5 py-0.5' : 'gap-1 px-1.5 py-0.5'
                  )}
                >
                  <Icon className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                  <span className="min-w-0 truncate font-medium text-foreground/90">
                    {item.value}
                  </span>
                </div>
              );
            })}
          </div>

          <div className="flex shrink-0 justify-end">
            <div
              id="detail-view-actions"
              className="inline-flex flex-wrap items-center justify-end gap-1 rounded-[14px] border border-border/60 bg-background/78 p-1 shadow-[0_6px_18px_rgba(15,23,42,0.04)]"
            >
              {visibleActions.map(renderActionButton)}

              {hasAiButtons ? (
                <>
                  {onTranslate && labels.translate ? (
                    <Button
                      type="button"
                      variant="outline"
                      size="icon"
                      onClick={onTranslate}
                      aria-label={labels.translate}
                      title={labels.translate}
                      className={iconButtonClassName}
                    >
                      <Languages className="h-4 w-4" />
                    </Button>
                  ) : null}
                  {onOpenChat && labels.chat ? (
                    <Button
                      type="button"
                      variant="outline"
                      size="icon"
                      onClick={onOpenChat}
                      aria-label={labels.chat}
                      title={labels.chat}
                      className={iconButtonClassName}
                    >
                      <MessageSquareText className="h-4 w-4" />
                    </Button>
                  ) : null}
                </>
              ) : null}

              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    aria-label={labels.moreActions}
                    title={labels.moreActions}
                    className={cn(
                      iconButtonClassName,
                      'border border-transparent text-muted-foreground hover:border-border/70 hover:bg-background hover:text-foreground'
                    )}
                  >
                    <MoreHorizontal className="h-4 w-4" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  <DropdownMenuItem onClick={onToggleFavorite} className="gap-2">
                    <Heart className="h-4 w-4" fill={entry.is_favorite ? 'currentColor' : 'none'} />
                    <span>{entry.is_favorite ? labels.unfavorite : labels.favorite}</span>
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          </div>
        </div>
      </CardHeader>

      <CardContent
        id="detail-view-content"
        className={cn(
          'flex min-h-0 flex-1 flex-col',
          hasImmersivePreview ? 'p-1 min-[1200px]:p-1.5' : 'p-1 min-[1200px]:p-1.5'
        )}
      >
        <div
          id="detail-view-shell"
          className={cn(
            'isolate flex h-full min-h-0 flex-1 flex-col overflow-hidden border border-border/70 bg-background/78',
            hasImmersivePreview
              ? 'rounded-[12px] min-[1200px]:rounded-[14px]'
              : 'rounded-[12px] min-[1200px]:rounded-[14px]'
          )}
        >
          <div
            id="detail-view-content-wrapper"
            className={cn(
              'flex min-h-0 flex-1 flex-col items-stretch overflow-y-auto overscroll-contain',
              hasImmersivePreview
                ? 'gap-1 p-1 min-[1200px]:gap-1.5 min-[1200px]:p-1.5'
                : 'gap-1.5 p-1 min-[1200px]:gap-2 min-[1200px]:p-1.5'
            )}
          >
            <div
              id="detail-view-primary-column"
              className={cn('shrink-0', hasImmersivePreview ? 'space-y-1' : 'space-y-1.5 pr-0.5')}
            >
              <PrimaryPreviewRenderer
                kind={descriptor.primaryKind}
                payload={descriptor.primaryPayload}
                onOpenFile={onOpenFile}
              />
              {showAlternateViews && <AlternateViews views={descriptor.alternateViews} />}
            </div>

            {hasInspector && (
              <div id="detail-view-inspector" className="shrink-0 pr-0.5">
                <InspectorPanel sections={descriptor.inspectorSections} />
              </div>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

export function DetailEmptyState({
  selectItemLabel,
  helperPills = [],
}: {
  selectItemLabel: string;
  helperPills?: string[];
}) {
  return (
    <Card
      id="detail-view-empty"
      className="flex h-full min-h-[220px] flex-col rounded-[18px] border border-border/70 bg-card/88 shadow-[0_16px_50px_rgba(15,23,42,0.08)] backdrop-blur-xl min-[1200px]:min-h-[240px] min-[1200px]:rounded-[20px]"
    >
      <CardContent
        id="detail-view-empty-content"
        className="flex flex-1 items-start justify-start p-4 min-[1200px]:p-5"
      >
        <div id="detail-view-empty-message" className="flex w-full max-w-[560px] flex-col gap-2.5">
          <div className="rounded-[16px] border border-border/70 bg-background/65 p-3.5 min-[1200px]:rounded-[18px] min-[1200px]:p-4">
            <div className="mb-3 flex h-12 w-12 items-center justify-center rounded-[14px] border border-primary/15 bg-primary/10 text-primary min-[1200px]:h-14 min-[1200px]:w-14 min-[1200px]:rounded-[16px]">
              <AppWindow className="h-7 w-7" />
            </div>
            <p className="text-[14px] font-medium text-foreground min-[1200px]:text-[15px]">
              {selectItemLabel}
            </p>
            <p className="mt-1.5 text-[12px] text-muted-foreground">Alt + 1-9</p>
          </div>

          {helperPills.length > 0 && (
            <div className="flex flex-wrap gap-1">
              {helperPills.map((pill) => (
                <Badge
                  key={pill}
                  variant="secondary"
                  className="rounded-full border border-border/60 bg-background/78 px-1.5 py-0.5 text-[10px] font-normal text-muted-foreground"
                >
                  {pill}
                </Badge>
              ))}
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
