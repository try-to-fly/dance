import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
  AppWindow,
  ClipboardPaste,
  Clock3,
  Copy,
  FileImage,
  FileText,
  FolderClosed,
  Heart,
  Layers3,
  Trash2,
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Badge } from '../ui/badge';
import { Button } from '../ui/button';
import { useClipboardStore } from '../../stores/clipboardStore';
import { ContentSubType, ContentMetadata } from '../../types/clipboard';
import {
  UnifiedTextRenderer,
  UrlRenderer,
  ColorRenderer,
  IpRenderer,
  EmailRenderer,
  TimeRenderer,
  JsonRenderer,
} from './ContentRenderers';
import { ImagePreview } from './ImagePreview';
import { cn } from '../../lib/utils';

const parseMetadata = (metadataString?: string | null): ContentMetadata | null => {
  if (!metadataString) return null;
  try {
    return JSON.parse(metadataString) as ContentMetadata;
  } catch {
    return null;
  }
};

const getFileName = (value?: string | null) => {
  if (!value) return '';
  return value.split(/[\\/]/).pop() || value;
};

const normalizeContentPreview = (value?: string | null, maxLength = 120) => {
  if (!value) return '';

  const normalized = value.replace(/\s+/g, ' ').trim();
  if (!normalized) return '';

  return normalized.length > maxLength
    ? `${normalized.slice(0, maxLength).trimEnd()}…`
    : normalized;
};

const editorSubTypes = new Set<ContentSubType>([
  'plain_text',
  'code',
  'command',
  'markdown',
  'json',
]);

const normalizeMetaLabel = (value: string) => value.replace(/[：:]$/, '');

export function DetailView() {
  const { t } = useTranslation(['common', 'clipboard']);
  const {
    selectedEntry,
    getImageUrl,
    openFileWithSystem,
    copyToClipboard,
    pasteSelectedEntry,
    toggleFavorite,
    deleteEntry,
  } = useClipboardStore();
  const [imageUrl, setImageUrl] = useState('');
  const [contentSubType, setContentSubType] = useState<ContentSubType>('plain_text');

  useEffect(() => {
    const loadImage = async () => {
      if (selectedEntry?.file_path && selectedEntry.content_type.toLowerCase().includes('image')) {
        try {
          const url = await getImageUrl(selectedEntry.file_path);
          setImageUrl(url);
        } catch (error) {
          console.error('[DetailView] 图片加载失败:', error);
          setImageUrl('');
        }
      } else {
        setImageUrl('');
      }
    };

    loadImage();

    if (selectedEntry?.content_subtype) {
      setContentSubType(selectedEntry.content_subtype as ContentSubType);
    } else {
      setContentSubType('plain_text');
    }
  }, [selectedEntry, getImageUrl]);

  const formatDate = (timestamp: number, compact = false) => {
    const date = new Date(timestamp);
    return date.toLocaleString(undefined, {
      year: compact ? undefined : 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: compact ? undefined : '2-digit',
    });
  };

  const getContentType = () => {
    if (!selectedEntry) return t('detail.unknown');
    const type = selectedEntry.content_type.toLowerCase();

    if (type.includes('text') || type.includes('string')) {
      return t(`detail.contentTypes.${contentSubType}`) || t('detail.contentTypes.text');
    }

    if (type.includes('image')) return t('detail.contentTypes.image');
    if (type.includes('file')) return t('detail.contentTypes.file');
    return t('detail.unknown');
  };

  const getHeadline = () => {
    if (!selectedEntry) return t('detail.title');

    if (selectedEntry.content_type.toLowerCase().includes('image')) {
      return getFileName(selectedEntry.file_path) || t('detail.contentTypes.image');
    }

    if (selectedEntry.content_type.toLowerCase().includes('file')) {
      return (
        getFileName(selectedEntry.file_path || selectedEntry.content_data) ||
        t('detail.contentTypes.file')
      );
    }

    if (contentSubType === 'url' && selectedEntry.content_data) {
      try {
        const url = new URL(selectedEntry.content_data);
        const path = url.pathname && url.pathname !== '/' ? url.pathname : '';
        return `${url.host}${path}` || selectedEntry.content_data;
      } catch {
        return normalizeContentPreview(selectedEntry.content_data) || getContentType();
      }
    }

    return normalizeContentPreview(selectedEntry.content_data) || getContentType();
  };

  const handleCopy = async () => {
    if (selectedEntry?.content_data) {
      await copyToClipboard(selectedEntry.content_data);
    }
  };

  const handlePaste = async () => {
    if (selectedEntry) {
      await pasteSelectedEntry(selectedEntry);
    }
  };

  const handleDelete = async () => {
    if (selectedEntry) {
      await deleteEntry(selectedEntry.id);
    }
  };

  const handleImageClick = async () => {
    if (selectedEntry?.file_path) {
      try {
        await openFileWithSystem(selectedEntry.file_path);
      } catch (error) {
        console.error('Failed to open image with system viewer:', error);
      }
    }
  };

  const metadataPills = selectedEntry
    ? [
        {
          key: 'source',
          icon: AppWindow,
          label: normalizeMetaLabel(t('detail.source')),
          value: selectedEntry.source_app || t('detail.unknown'),
          fullValue: selectedEntry.source_app || t('detail.unknown'),
        },
        {
          key: 'time',
          value: formatDate(selectedEntry.created_at, true),
          fullValue: formatDate(selectedEntry.created_at),
          icon: Clock3,
          label: normalizeMetaLabel(t('detail.time')),
        },
        {
          key: 'copy-count',
          icon: Layers3,
          label: normalizeMetaLabel(t('detail.copyCount')),
          value: String(selectedEntry.copy_count),
          fullValue: String(selectedEntry.copy_count),
        },
      ]
    : [];

  const renderContent = () => {
    if (!selectedEntry) return null;

    if (selectedEntry.content_type.toLowerCase().includes('image')) {
      const metadata = parseMetadata(selectedEntry.metadata);
      const imageMetadata = metadata?.image_metadata;

      if (imageUrl && selectedEntry.file_path) {
        return (
          <ImagePreview
            imageUrl={imageUrl}
            filePath={selectedEntry.file_path}
            metadata={imageMetadata}
            onOpenWithSystem={handleImageClick}
          />
        );
      }

      return (
        <div className="flex min-h-[320px] flex-1 flex-col items-center justify-center p-8 text-center">
          <div className="mb-4 h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent" />
          <p className="text-sm text-muted-foreground">{t('detail.loading')}</p>
          {selectedEntry.file_path && (
            <p className="mt-3 break-all text-xs text-muted-foreground">
              {selectedEntry.file_path}
            </p>
          )}
        </div>
      );
    }

    if (selectedEntry.content_type.toLowerCase().includes('file')) {
      return (
        <div className="flex min-h-[320px] flex-1 flex-col items-center justify-center p-8 text-center">
          <div className="mb-5 flex h-16 w-16 items-center justify-center rounded-[22px] border border-primary/15 bg-primary/10 text-primary">
            <FolderClosed className="h-8 w-8" />
          </div>
          <p className="max-w-xl break-all text-sm text-foreground">
            {selectedEntry.file_path || selectedEntry.content_data}
          </p>
        </div>
      );
    }

    const content = selectedEntry.content_data || '';
    const metadata = selectedEntry.metadata;

    switch (contentSubType) {
      case 'url':
        return <UrlRenderer content={content} metadata={metadata} />;
      case 'ip_address':
        return <IpRenderer content={content} />;
      case 'email':
        return <EmailRenderer content={content} />;
      case 'color':
        return <ColorRenderer content={content} metadata={metadata} />;
      case 'timestamp':
        return <TimeRenderer content={content} metadata={metadata} />;
      case 'json':
        return <JsonRenderer content={content} />;
      case 'code':
      case 'markdown':
      case 'command':
      case 'plain_text':
      default:
        return (
          <UnifiedTextRenderer
            content={content}
            contentSubType={contentSubType}
            metadata={metadata}
          />
        );
    }
  };

  if (!selectedEntry) {
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
            <p className="text-base font-medium text-foreground">{t('detail.selectItem')}</p>
            <p className="mt-2 text-sm text-muted-foreground">Alt + 1-9</p>
          </div>
        </CardContent>
      </Card>
    );
  }

  const contentIsImage = selectedEntry.content_type.toLowerCase().includes('image');
  const contentIsFile = selectedEntry.content_type.toLowerCase().includes('file');
  const headline = getHeadline();
  const contentUsesEditorLayout =
    !contentIsImage && !contentIsFile && editorSubTypes.has(contentSubType);
  const contentTypeLabel = getContentType();

  return (
    <Card
      id="detail-view"
      className="flex h-full min-h-0 flex-col overflow-hidden rounded-[22px] border border-border/70 bg-card/88 shadow-[0_16px_50px_rgba(15,23,42,0.08)] backdrop-blur-xl min-[1200px]:rounded-[26px]"
    >
      <CardHeader
        id="detail-view-header"
        className="gap-3 space-y-0 border-b border-border/70 px-4 pb-3 pt-4 min-[1200px]:gap-3.5 min-[1200px]:px-5 min-[1200px]:pb-4 min-[1200px]:pt-5"
      >
        <div className="flex flex-wrap items-start justify-between gap-3 min-[1200px]:gap-4">
          <div className="min-w-0 flex-1 space-y-2">
            <div className="flex flex-wrap items-center gap-2">
              <Badge
                id="detail-view-type-badge"
                variant="secondary"
                className="rounded-full border border-primary/20 bg-primary/10 px-2.5 py-1 text-[11px] text-primary min-[1200px]:px-3 min-[1200px]:text-sm"
              >
                {contentIsImage ? (
                  <FileImage className="mr-2 h-3.5 w-3.5" />
                ) : contentIsFile ? (
                  <FolderClosed className="mr-2 h-3.5 w-3.5" />
                ) : (
                  <FileText className="mr-2 h-3.5 w-3.5" />
                )}
                {contentTypeLabel}
              </Badge>

              {selectedEntry.is_favorite && (
                <Badge
                  variant="outline"
                  className="rounded-full border-amber-200 bg-amber-50 px-2.5 py-1 text-[11px] text-amber-700 dark:border-amber-400/20 dark:bg-amber-400/10 dark:text-amber-300 min-[1200px]:px-3 min-[1200px]:text-sm"
                >
                  <Heart className="mr-2 h-3.5 w-3.5" fill="currentColor" />
                  {t('clipboard:actions.favorite')}
                </Badge>
              )}
            </div>

            <div className="space-y-1">
              <p className="text-[10px] font-semibold uppercase tracking-[0.26em] text-muted-foreground min-[1200px]:text-[11px]">
                {t('detail.title')}
              </p>
              <CardTitle
                id="detail-view-title"
                className="line-clamp-2 text-base leading-snug tracking-tight min-[1200px]:text-lg"
                title={headline}
              >
                {headline}
              </CardTitle>
            </div>
          </div>

          <div className="flex shrink-0 flex-wrap items-center gap-1.5 min-[1200px]:gap-2">
            <Button
              type="button"
              variant="secondary"
              onClick={handleCopy}
              disabled={!selectedEntry.content_data}
              aria-label={t('copy')}
              className="h-8 rounded-xl px-2.5 min-[1200px]:h-9 min-[1200px]:rounded-2xl min-[1200px]:px-4"
            >
              <Copy className="h-4 w-4 min-[1200px]:mr-2" />
              <span className="hidden min-[1200px]:inline">{t('copy')}</span>
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={handlePaste}
              aria-label={t('paste')}
              className="h-8 rounded-xl px-2.5 min-[1200px]:h-9 min-[1200px]:rounded-2xl min-[1200px]:px-4"
            >
              <ClipboardPaste className="h-4 w-4 min-[1200px]:mr-2" />
              <span className="hidden min-[1200px]:inline">{t('paste')}</span>
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={() => toggleFavorite(selectedEntry.id)}
              aria-label={
                selectedEntry.is_favorite
                  ? t('clipboard:actions.unfavorite')
                  : t('clipboard:actions.favorite')
              }
              className={cn(
                'h-8 rounded-xl px-2.5 min-[1200px]:h-9 min-[1200px]:rounded-2xl min-[1200px]:px-4',
                selectedEntry.is_favorite && 'border-primary/30 bg-primary/10 text-primary'
              )}
            >
              <Heart
                className="h-4 w-4 min-[1200px]:mr-2"
                fill={selectedEntry.is_favorite ? 'currentColor' : 'none'}
              />
              <span className="hidden min-[1200px]:inline">
                {selectedEntry.is_favorite
                  ? t('clipboard:actions.unfavorite')
                  : t('clipboard:actions.favorite')}
              </span>
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={handleDelete}
              aria-label={t('delete')}
              className="h-8 rounded-xl px-2.5 text-destructive hover:text-destructive min-[1200px]:h-9 min-[1200px]:rounded-2xl min-[1200px]:px-4"
            >
              <Trash2 className="h-4 w-4 min-[1200px]:mr-2" />
              <span className="hidden min-[1200px]:inline">{t('delete')}</span>
            </Button>
          </div>
        </div>

        <div id="detail-view-metadata" className="flex flex-wrap items-center gap-2">
          {metadataPills.map((item) => {
            const Icon = item.icon;

            return (
              <div
                key={item.key}
                title={`${item.label}: ${item.fullValue}`}
                className="inline-flex min-w-0 max-w-full items-center gap-2 rounded-full border border-border/70 bg-background/70 px-2.5 py-1.5 text-xs min-[1200px]:px-3 min-[1200px]:py-2"
              >
                <Icon className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                <span className="shrink-0 text-[11px] text-muted-foreground min-[1200px]:text-xs">
                  {item.label}
                </span>
                <span className="min-w-0 truncate font-medium text-foreground">{item.value}</span>
              </div>
            );
          })}
        </div>
      </CardHeader>

      <CardContent
        id="detail-view-content"
        className="flex min-h-0 flex-1 flex-col p-3 min-[1200px]:p-5"
      >
        <div
          id="detail-view-shell"
          className={cn(
            'flex h-full min-h-0 min-h-[320px] flex-1 flex-col rounded-[20px] border border-border/70 bg-background/70 min-[1200px]:min-h-[360px] min-[1200px]:rounded-[24px]',
            contentUsesEditorLayout ? 'overflow-hidden' : 'overflow-y-auto'
          )}
        >
          <div
            id="detail-view-content-wrapper"
            className={cn(
              'min-h-0 flex-1',
              contentUsesEditorLayout
                ? 'flex h-full min-h-0 flex-col overflow-hidden'
                : 'p-3.5 min-[1200px]:p-5'
            )}
          >
            {renderContent()}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
