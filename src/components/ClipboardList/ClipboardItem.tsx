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
import { ClipboardEntry } from '../../types/clipboard';
import { useClipboardStore } from '../../stores/clipboardStore';
import { cn } from '../../lib/utils';
import { buildPreviewSummary } from '../../lib/preview/previewSummary';

interface ClipboardItemProps {
  entry: ClipboardEntry;
  isSelected?: boolean;
  onClick?: () => void;
  showNumber?: boolean;
  number?: number;
}

export const ClipboardItem: React.FC<ClipboardItemProps> = ({
  entry,
  isSelected,
  onClick,
  showNumber,
  number,
}) => {
  const { t } = useTranslation(['common', 'clipboard']);
  const {
    toggleFavorite,
    deleteEntry,
    copyToClipboard,
    pasteSelectedEntry,
    getAppIcon,
  } = useClipboardStore();
  const [appIconUrl, setAppIconUrl] = useState<string | null>(null);
  const summary = buildPreviewSummary(entry, 'list');
  const usesWorkbenchSummary =
    summary.previewIntent === 'code_workbench' || summary.previewIntent === 'command_workbench';
  const usesStructuredMonoSummary =
    summary.previewIntent === 'json_structured' || summary.previewIntent === 'base64_summary';

  useEffect(() => {
    if (entry.app_bundle_id && getAppIcon) {
      getAppIcon(entry.app_bundle_id)
        .then(setAppIconUrl)
        .catch(() => setAppIconUrl(null));
      return;
    }

    setAppIconUrl(null);
  }, [entry.app_bundle_id, getAppIcon]);

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

  const { label: typeLabel } = getTypeMeta();

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
          className={cn(
            'group relative h-[118px] cursor-pointer overflow-hidden rounded-[18px] border border-border/70 bg-background/72 px-3.5 py-3 shadow-[0_8px_24px_rgba(15,23,42,0.06)] transition-all duration-200 hover:border-primary/20 hover:bg-background/88 hover:shadow-[0_16px_36px_rgba(15,23,42,0.1)] min-[1200px]:h-[126px] min-[1200px]:rounded-[20px] min-[1200px]:px-4 min-[1200px]:py-3.5',
            {
              'border-primary/30 bg-primary/8 shadow-[0_18px_42px_rgba(13,148,136,0.14)] ring-1 ring-primary/15':
                isSelected,
            }
          )}
          onClick={onClick}
          onDoubleClick={handlePaste}
        >
          <div className="absolute inset-y-3 left-0 w-1 rounded-r-full bg-primary/70 opacity-0 transition-opacity duration-200 group-hover:opacity-60 min-[1200px]:inset-y-3.5" />
          {isSelected && (
            <div className="absolute inset-y-3 left-0 w-1 rounded-r-full bg-primary min-[1200px]:inset-y-3.5" />
          )}

          <div className="flex h-full min-w-0 flex-col gap-3">
            <div className="flex items-start justify-between gap-3">
              <div className="flex min-w-0 flex-wrap items-center gap-1.5 min-[1200px]:gap-2">
                <Badge
                  variant="secondary"
                  className={cn(
                    'rounded-full border px-2.5 py-0.5 text-[10px] font-medium min-[1200px]:text-[11px]',
                    isSelected
                      ? 'border-primary/20 bg-primary/10 text-primary'
                      : 'border-border/70 bg-secondary/70 text-foreground'
                  )}
                >
                  {typeLabel}
                </Badge>

                <span className="inline-flex items-center gap-1.5 rounded-full bg-secondary/70 px-2.5 py-0.5 text-[11px] text-muted-foreground min-[1200px]:text-xs">
                  <Clock3 className="h-3.5 w-3.5" />
                  <span>{formatDate(entry.created_at)}</span>
                </span>

                {entry.source_app && (
                  <span className="inline-flex max-w-full items-center gap-1.5 rounded-full bg-secondary/70 px-2.5 py-0.5 text-[11px] text-muted-foreground min-[1200px]:text-xs">
                    {appIconUrl ? (
                      <img src={appIconUrl} alt={entry.source_app} className="h-4 w-4 rounded-sm" />
                    ) : (
                      <span className="flex h-4 w-4 items-center justify-center rounded-sm bg-background/80 text-[9px] font-semibold uppercase text-foreground">
                        {entry.source_app.charAt(0)}
                      </span>
                    )}
                    <span className="max-w-[120px] truncate min-[1200px]:max-w-[160px]">
                      {entry.source_app}
                    </span>
                  </span>
                )}

                {entry.copy_count > 1 && (
                  <Badge
                    variant="outline"
                    className="rounded-full border-border/70 px-2.5 py-0.5 text-[11px]"
                  >
                    {t('clipboard:actions.copiedTimes', { count: entry.copy_count })}
                  </Badge>
                )}
              </div>

              <div className="flex shrink-0 items-center gap-1.5 min-[1200px]:gap-2">
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
                    className="h-6 w-6 rounded-full p-0 text-[10px] font-semibold shadow-sm min-[1200px]:h-7 min-[1200px]:w-7 min-[1200px]:text-[11px]"
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
                        'h-7 w-7 rounded-full border border-transparent bg-transparent text-muted-foreground transition-all duration-200 min-[1200px]:h-8 min-[1200px]:w-8',
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

            <div className="flex min-h-0 flex-1 overflow-hidden rounded-[18px] border border-border/60 bg-secondary/20 px-4 py-3">
              <div className="flex min-w-0 flex-1 flex-col justify-between gap-2">
                <p
                  className={cn(
                    'line-clamp-1 text-sm font-semibold leading-5 text-foreground',
                    (usesWorkbenchSummary || usesStructuredMonoSummary) && 'font-mono text-[13px]'
                  )}
                >
                  {summary.headline}
                </p>

                <p
                  className={cn(
                    'line-clamp-2 text-xs leading-5 text-muted-foreground',
                    usesWorkbenchSummary && 'font-mono',
                    usesStructuredMonoSummary && 'font-mono text-[11px]'
                  )}
                >
                  {summary.secondarySummary}
                </p>
              </div>
            </div>
          </div>
        </Card>
      </ContextMenuTrigger>

      <ContextMenuContent>{menuContent}</ContextMenuContent>
    </ContextMenu>
  );
};
