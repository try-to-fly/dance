import React, { useEffect, useState } from 'react';
import { format } from 'date-fns';
import { useTranslation } from 'react-i18next';
import {
  Binary,
  Braces,
  ClipboardPaste,
  Clock3,
  Code2,
  Copy,
  FileImage,
  FileText,
  FolderClosed,
  Globe2,
  Link2,
  Mail,
  MoreVertical,
  Palette,
  Star,
  TerminalSquare,
  Trash2,
} from 'lucide-react';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from '../ui/context-menu';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '../ui/dropdown-menu';
import {
  ClipboardEntry,
  ClipboardRetrievalMatchKind,
  PreviewSummaryDensity,
} from '../../types/clipboard';
import { useClipboardStore } from '../../stores/clipboardStore';
import { cn } from '../../lib/utils';
import {
  getEntryPresentationMetadata,
  normalizeContentPreview,
} from '../../lib/preview/entryPresentation';
import { buildPreviewSummary } from '../../lib/preview/previewSummary';

interface ClipboardItemProps {
  entry: ClipboardEntry;
  isSelected?: boolean;
  onClick?: () => void;
  showNumber?: boolean;
  number?: number;
  density?: PreviewSummaryDensity;
  activeFilterReasons?: string[];
}

const RETRIEVAL_REASON_LABELS: Record<ClipboardRetrievalMatchKind, string> = {
  content: '文本',
  source_app: '来源应用',
  url_host: 'URL Host',
  url_path: 'URL Path',
  url_query: 'URL Path',
  json_key: 'JSON Key',
  command_name: '命令名',
  color_value: '颜色格式',
  metadata: '文本',
  fuzzy: '模糊匹配',
};

const buildRetrievalReasons = (entry: ClipboardEntry, activeFilterReasons: string[]) => {
  const reasons = [
    ...(entry.retrieval?.match_kind
      ? [RETRIEVAL_REASON_LABELS[entry.retrieval.match_kind] ?? entry.retrieval.label]
      : []),
    ...activeFilterReasons,
  ];

  return Array.from(new Set(reasons.filter(Boolean)));
};

const formatImageFileSize = (bytes?: number) => {
  if (typeof bytes !== 'number' || !Number.isFinite(bytes) || bytes <= 0) {
    return '';
  }

  const units = ['B', 'KB', 'MB', 'GB'];
  let value = bytes;
  let unitIndex = 0;

  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }

  const fractionDigits = value >= 10 || unitIndex === 0 ? 0 : 1;
  return `${Number(value.toFixed(fractionDigits))} ${units[unitIndex]}`;
};

export const ClipboardItem: React.FC<ClipboardItemProps> = ({
  entry,
  isSelected,
  onClick,
  showNumber,
  number,
  density = 'list',
  activeFilterReasons = [],
}) => {
  const { t } = useTranslation(['common', 'clipboard']);
  const {
    toggleFavorite,
    deleteEntry,
    copyToClipboard,
    pasteSelectedEntry,
    getImageUrl,
    getAppIcon,
    resolveEntryPreview,
  } = useClipboardStore();
  const [appIconUrl, setAppIconUrl] = useState<string | null>(null);
  const [imagePreviewUrl, setImagePreviewUrl] = useState<string | null>(null);
  const summary = buildPreviewSummary(entry, density);
  const usesWorkbenchSummary =
    summary.previewIntent === 'code_workbench' || summary.previewIntent === 'command_workbench';
  const usesStructuredMonoSummary =
    summary.previewIntent === 'json_structured' || summary.previewIntent === 'base64_summary';
  const retrievalSnippet = entry.retrieval?.snippet?.trim() || '';
  const retrievalReasons = buildRetrievalReasons(entry, activeFilterReasons);
  const visibleRetrievalReasons = retrievalReasons.slice(0, 2);
  const overflowRetrievalReasons = retrievalReasons.length - visibleRetrievalReasons.length;
  const isRetrievalDensity = density === 'retrieval';
  const getTypeMeta = () => {
    switch (summary.semanticType) {
      case 'image':
        return { Icon: FileImage, label: t('clipboard:contentTypes.image') };
      case 'file':
        return { Icon: FolderClosed, label: t('clipboard:contentTypes.file') };
      case 'url':
        return { Icon: Link2, label: t('clipboard:contentTypes.url') };
      case 'ip_address':
        return { Icon: Globe2, label: t('clipboard:contentTypes.ipAddress') };
      case 'email':
        return { Icon: Mail, label: t('clipboard:contentTypes.email') };
      case 'color':
        return { Icon: Palette, label: t('clipboard:contentTypes.color') };
      case 'code':
        return { Icon: Code2, label: t('clipboard:contentTypes.codeSnippet') };
      case 'command':
        return { Icon: TerminalSquare, label: t('clipboard:contentTypes.command') };
      case 'timestamp':
        return { Icon: Clock3, label: t('clipboard:contentTypes.timestamp') };
      case 'json':
        return { Icon: Braces, label: t('clipboard:contentTypes.json') };
      case 'markdown':
        return { Icon: FileText, label: t('clipboard:contentTypes.markdown') };
      case 'base64': {
        const translated = t('clipboard:contentTypes.base64');
        return {
          Icon: Binary,
          label:
            translated === 'clipboard:contentTypes.base64' || translated === 'contentTypes.base64'
              ? 'Base64'
              : translated,
        };
      }
      case 'plain_text':
      default:
        return { Icon: FileText, label: t('clipboard:contentTypes.plainText') };
    }
  };
  const { label: typeLabel } = getTypeMeta();
  const isImageEntry = summary.semanticType === 'image';
  const imageMetadata = isImageEntry ? getEntryPresentationMetadata(entry)?.image_metadata : null;
  const imageResolution =
    imageMetadata?.width && imageMetadata?.height
      ? `${imageMetadata.width} x ${imageMetadata.height}`
      : typeLabel;
  const imageFileSize = formatImageFileSize(imageMetadata?.file_size);
  const inlineImageUrl =
    isImageEntry && entry.content_data?.startsWith('data:image/') ? entry.content_data : null;
  const imagePreviewMeta = imageFileSize || typeLabel;

  useEffect(() => {
    if (entry.app_bundle_id && getAppIcon) {
      getAppIcon(entry.app_bundle_id)
        .then(setAppIconUrl)
        .catch(() => setAppIconUrl(null));
      return;
    }

    setAppIconUrl(null);
  }, [entry.app_bundle_id, getAppIcon]);

  useEffect(() => {
    let cancelled = false;

    if (!isImageEntry) {
      setImagePreviewUrl(null);
      return;
    }

    if (inlineImageUrl) {
      setImagePreviewUrl(inlineImageUrl);
      return;
    }

    if (!entry.file_path) {
      setImagePreviewUrl(null);
      return;
    }

    setImagePreviewUrl(null);
    const filePath = entry.file_path;

    const loadImagePreview = async () => {
      try {
        const resolved = resolveEntryPreview ? await resolveEntryPreview(entry) : null;
        const nextUrl = resolved?.imageUrl || (await getImageUrl(filePath));

        if (!cancelled) {
          setImagePreviewUrl(nextUrl || null);
        }
      } catch {
        if (!cancelled) {
          setImagePreviewUrl(null);
        }
      }
    };

    void loadImagePreview();

    return () => {
      cancelled = true;
    };
  }, [entry, entry.file_path, getImageUrl, inlineImageUrl, isImageEntry, resolveEntryPreview]);

  const formatDate = (timestamp: number) => {
    return format(new Date(timestamp), 'MM-dd HH:mm');
  };

  const handleCopy = async () => {
    if (entry.content_data) {
      await copyToClipboard(entry.content_data);
    }
  };

  const handlePaste = async () => {
    if (pasteSelectedEntry) {
      await pasteSelectedEntry(entry);
    }
  };

  const rawPreviewFallback = normalizeContentPreview(entry.content_data, 160);
  const previewHeadline = summary.headline.trim() || rawPreviewFallback || typeLabel;
  const previewSecondary = summary.secondarySummary.trim() || rawPreviewFallback || previewHeadline;

  const menuContent = (
    <>
      <ContextMenuItem className="flex items-center gap-2" onClick={handlePaste}>
        <ClipboardPaste className="h-4 w-4" />
        <span>{t('common:paste')}</span>
      </ContextMenuItem>
      <ContextMenuItem className="flex items-center gap-2" onClick={handleCopy}>
        <Copy className="h-4 w-4" />
        <span>{t('common:copy')}</span>
      </ContextMenuItem>
      <ContextMenuItem className="flex items-center gap-2" onClick={() => toggleFavorite(entry.id)}>
        <Star className="h-4 w-4" fill={entry.is_favorite ? 'currentColor' : 'none'} />
        <span>
          {entry.is_favorite ? t('clipboard:actions.unfavorite') : t('clipboard:actions.favorite')}
        </span>
      </ContextMenuItem>
      <ContextMenuSeparator />
      <ContextMenuItem
        className="flex items-center gap-2 text-destructive focus:text-destructive"
        onClick={() => deleteEntry(entry.id)}
      >
        <Trash2 className="h-4 w-4" />
        <span>{t('common:delete')}</span>
      </ContextMenuItem>
    </>
  );

  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>
        <Card
          data-semantic-type={summary.semanticType}
          data-preview-intent={summary.previewIntent}
          data-density={density}
          className={cn(
            'group relative min-h-[100px] cursor-pointer overflow-hidden rounded-[14px] border border-border/70 bg-background/76 px-2 py-1.5 shadow-[0_6px_18px_rgba(15,23,42,0.045)] transition-all duration-200 hover:border-primary/20 hover:bg-background/92 hover:shadow-[0_12px_24px_rgba(15,23,42,0.08)] min-[1200px]:min-h-[104px] min-[1200px]:rounded-[16px] min-[1200px]:px-2.5 min-[1200px]:py-2',
            isRetrievalDensity && 'min-h-[122px] min-[1200px]:min-h-[128px]',
            {
              'border-primary/30 bg-primary/8 shadow-[0_18px_42px_rgba(13,148,136,0.14)] ring-1 ring-primary/15':
                isSelected,
            }
          )}
          onClick={onClick}
          onDoubleClick={handlePaste}
        >
          <div className="absolute inset-y-2 left-0 w-1 rounded-r-full bg-primary/70 opacity-0 transition-opacity duration-200 group-hover:opacity-60 min-[1200px]:inset-y-2.5" />
          {isSelected && (
            <div className="absolute inset-y-2 left-0 w-1 rounded-r-full bg-primary min-[1200px]:inset-y-2.5" />
          )}

          <div
            className={cn(
              'grid h-full min-w-0 gap-1',
              isRetrievalDensity
                ? 'grid-rows-[auto_auto_minmax(0,1fr)] gap-1.5'
                : 'grid-rows-[auto_minmax(38px,1fr)]'
            )}
          >
            <div className="flex items-start justify-between gap-1.5">
              <div className="flex min-w-0 items-center gap-1 overflow-hidden">
                <Badge
                  variant="secondary"
                  className={cn(
                    'shrink-0 rounded-full border px-1.5 py-0.5 text-[10px] font-medium',
                    isSelected
                      ? 'border-primary/20 bg-primary/10 text-primary'
                      : 'border-border/70 bg-secondary/70 text-foreground'
                  )}
                >
                  {typeLabel}
                </Badge>

                <span className="inline-flex shrink-0 items-center gap-1 rounded-full bg-secondary/70 px-1.5 py-0.5 text-[10px] text-muted-foreground">
                  <Clock3 className="h-3 w-3" />
                  <span>{formatDate(entry.created_at)}</span>
                </span>

                {entry.source_app && (
                  <span className="inline-flex min-w-0 max-w-[136px] items-center gap-1 overflow-hidden rounded-full bg-secondary/70 px-1.5 py-0.5 text-[10px] text-muted-foreground min-[1200px]:max-w-[150px]">
                    {appIconUrl ? (
                      <img
                        src={appIconUrl}
                        alt={entry.source_app}
                        className="h-3.5 w-3.5 rounded-sm"
                      />
                    ) : (
                      <span className="flex h-3.5 w-3.5 items-center justify-center rounded-sm bg-background/80 text-[8px] font-semibold uppercase text-foreground">
                        {entry.source_app.charAt(0)}
                      </span>
                    )}
                    <span className="max-w-[108px] truncate min-[1200px]:max-w-[132px]">
                      {entry.source_app}
                    </span>
                  </span>
                )}

                {entry.copy_count > 1 && (
                  <Badge
                    variant="outline"
                    className="shrink-0 rounded-full border-border/70 px-1.5 py-0.5 text-[10px]"
                  >
                    {t('clipboard:actions.copiedTimes', { count: entry.copy_count })}
                  </Badge>
                )}
              </div>

              <div className="flex shrink-0 items-center gap-1">
                {entry.is_favorite && (
                  <Star
                    className={cn(
                      'h-4 w-4',
                      isSelected ? 'text-primary' : 'text-amber-500 dark:text-amber-400'
                    )}
                    fill="currentColor"
                  />
                )}

                {showNumber && number && number <= 9 && (
                  <Badge
                    variant="default"
                    className="h-5 w-5 justify-center rounded-full p-0 text-center text-[10px] font-semibold leading-none shadow-sm min-[1200px]:h-[22px] min-[1200px]:w-[22px]"
                  >
                    {number}
                  </Badge>
                )}

                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button
                      variant="ghost"
                      size="icon"
                      aria-label="More actions"
                      className={cn(
                        'h-6 w-6 rounded-full border border-transparent bg-transparent text-muted-foreground transition-all duration-200',
                        isSelected
                          ? 'hover:border-primary/20 hover:bg-primary/10 hover:text-primary'
                          : 'hover:border-border/70 hover:bg-secondary/80 hover:text-foreground'
                      )}
                    >
                      <MoreVertical className="h-4 w-4" />
                    </Button>
                  </DropdownMenuTrigger>

                  <DropdownMenuContent align="end">
                    <DropdownMenuItem className="flex items-center gap-2" onClick={handlePaste}>
                      <ClipboardPaste className="h-4 w-4" />
                      <span>{t('common:paste')}</span>
                    </DropdownMenuItem>
                    <DropdownMenuItem className="flex items-center gap-2" onClick={handleCopy}>
                      <Copy className="h-4 w-4" />
                      <span>{t('common:copy')}</span>
                    </DropdownMenuItem>
                    <DropdownMenuItem
                      className="flex items-center gap-2"
                      onClick={() => toggleFavorite(entry.id)}
                    >
                      <Star
                        className="h-4 w-4"
                        fill={entry.is_favorite ? 'currentColor' : 'none'}
                      />
                      <span>
                        {entry.is_favorite
                          ? t('clipboard:actions.unfavorite')
                          : t('clipboard:actions.favorite')}
                      </span>
                    </DropdownMenuItem>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem
                      className="flex items-center gap-2 text-destructive focus:text-destructive"
                      onClick={() => deleteEntry(entry.id)}
                    >
                      <Trash2 className="h-4 w-4" />
                      <span>{t('common:delete')}</span>
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
            </div>

            <div
              className={cn(
                'flex min-h-[38px] overflow-hidden rounded-[12px] border border-border/60 bg-secondary/20 px-2 py-1.5 min-[1200px]:rounded-[14px]',
                isImageEntry && 'items-center gap-2 p-1.5'
              )}
            >
              {isImageEntry ? (
                <>
                  <div className="flex h-[54px] w-[72px] shrink-0 items-center justify-center overflow-hidden rounded-[10px] border border-border/60 bg-background/80">
                    {imagePreviewUrl ? (
                      <img
                        src={imagePreviewUrl}
                        alt={typeLabel}
                        className="h-full w-full object-cover"
                      />
                    ) : (
                      <div className="flex h-full w-full items-center justify-center bg-gradient-to-br from-primary/10 via-background/90 to-background">
                        <FileImage className="h-5 w-5 text-muted-foreground" />
                      </div>
                    )}
                  </div>

                  <div className="grid min-w-0 flex-1 content-center gap-0.5 overflow-hidden">
                    <div className="truncate text-[13px] font-semibold leading-[1.35] text-foreground">
                      {imageResolution}
                    </div>
                    <div className="truncate text-[10px] leading-[1.4] text-muted-foreground">
                      {imagePreviewMeta}
                    </div>
                  </div>
                </>
              ) : (
                <div className="grid min-w-0 flex-1 grid-rows-[auto_minmax(0,1fr)] gap-0.5 overflow-hidden">
                  <div
                    title={previewHeadline}
                    className={cn(
                      'min-w-0 truncate text-[13px] font-semibold leading-[1.35] text-foreground',
                      (usesWorkbenchSummary || usesStructuredMonoSummary) && 'font-mono text-[13px]'
                    )}
                  >
                    {previewHeadline}
                  </div>

                  <div
                    title={previewSecondary}
                    className={cn(
                      isRetrievalDensity
                        ? 'truncate text-[10px] leading-[1.4] text-muted-foreground'
                        : 'max-h-8 overflow-hidden break-words text-[10px] leading-[1.4] text-muted-foreground',
                      usesWorkbenchSummary && 'font-mono',
                      usesStructuredMonoSummary && 'font-mono text-[11px]'
                    )}
                  >
                    {previewSecondary}
                  </div>
                </div>
              )}
            </div>

            {isRetrievalDensity && (retrievalSnippet || visibleRetrievalReasons.length > 0) && (
              <div className="flex flex-col gap-1 rounded-[12px] border border-primary/10 bg-primary/[0.04] px-2 py-1.5">
                {retrievalSnippet ? (
                  <p
                    title={retrievalSnippet}
                    className={cn(
                      'line-clamp-2 break-words text-[10px] leading-[1.4] text-muted-foreground',
                      (usesWorkbenchSummary || usesStructuredMonoSummary) && 'font-mono'
                    )}
                  >
                    {retrievalSnippet}
                  </p>
                ) : null}

                {visibleRetrievalReasons.length > 0 && (
                  <div className="flex flex-wrap items-center gap-1">
                    {visibleRetrievalReasons.map((reason) => (
                      <Badge
                        key={reason}
                        variant="secondary"
                        className="rounded-full border border-primary/10 bg-background/80 px-1.5 py-0.5 text-[10px] font-normal text-muted-foreground"
                      >
                        {reason}
                      </Badge>
                    ))}

                    {overflowRetrievalReasons > 0 && (
                      <Badge
                        variant="secondary"
                        className="rounded-full border border-primary/10 bg-background/80 px-1.5 py-0.5 text-[10px] font-normal text-muted-foreground"
                      >
                        +{overflowRetrievalReasons}
                      </Badge>
                    )}
                  </div>
                )}
              </div>
            )}
          </div>
        </Card>
      </ContextMenuTrigger>

      <ContextMenuContent>{menuContent}</ContextMenuContent>
    </ContextMenu>
  );
};
