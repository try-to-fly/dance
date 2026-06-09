import { useState } from 'react';
import { File, FileCode2, Music, Video } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useExternalMediaDrag } from '../../../hooks/useExternalMediaDrag';
import {
  ClipboardEntry,
  ContentMetadata,
  ContentSubType,
  PreviewKind,
  ResolvedPreviewData,
} from '../../../types/clipboard';
import { Card, CardContent, CardHeader } from '../../ui/card';
import { Badge } from '../../ui/badge';
import { ImagePreview } from '../ImagePreview';
import {
  ColorRenderer,
  EmailRenderer,
  IpRenderer,
  JsonRenderer,
  TimeRenderer,
  UnifiedTextRenderer,
} from '../ContentRenderers';
import { UrlCardRenderer } from '../ContentRenderers/UrlCardRenderer';

interface PrimaryPayload {
  entry: ClipboardEntry;
  subType: ContentSubType;
  metadata: ContentMetadata | null;
  resolvedData?: ResolvedPreviewData;
  sessionKey?: string;
  onContentChange?: (value: string) => void;
}

interface PrimaryPreviewRendererProps {
  kind: PreviewKind;
  payload: unknown;
  onOpenFile: () => Promise<void> | void;
}

const getJsonText = (value: unknown): string => {
  if (typeof value === 'string') {
    return value;
  }

  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
};

const formatBinarySize = (bytes?: number | null) => {
  if (typeof bytes !== 'number' || !Number.isFinite(bytes) || bytes <= 0) {
    return '';
  }

  if (bytes < 1024) {
    return `${bytes} B`;
  }

  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }

  if (bytes < 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
};

const formatDurationFromSeconds = (seconds: number) => {
  if (!Number.isFinite(seconds) || seconds <= 0) {
    return '';
  }

  const totalSeconds = Math.round(seconds);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const remainingSeconds = totalSeconds % 60;

  if (hours > 0) {
    return `${hours}:${String(minutes).padStart(2, '0')}:${String(remainingSeconds).padStart(2, '0')}`;
  }

  return `${minutes}:${String(remainingSeconds).padStart(2, '0')}`;
};

const formatMediaFormat = (value?: string | null) => {
  if (!value) {
    return '';
  }

  return value.replace(/^\./, '').toUpperCase();
};

const buildMediaInfoItems = ({
  resolvedData,
  browserMetadata,
  includeSampleRate = false,
}: {
  resolvedData?: ResolvedPreviewData;
  browserMetadata?: {
    width?: number;
    height?: number;
    duration?: string;
  };
  includeSampleRate?: boolean;
}) => {
  const media = resolvedData?.media;
  const width = media?.width ?? browserMetadata?.width;
  const height = media?.height ?? browserMetadata?.height;
  const size =
    media?.size ||
    formatBinarySize(
      media?.sizeBytes ?? resolvedData?.sizeBytes ?? resolvedData?.url?.contentLength
    );
  const mime = resolvedData?.mime || resolvedData?.url?.contentType;
  const format = formatMediaFormat(media?.format ?? resolvedData?.extension);

  return [
    width && height ? { label: 'Resolution', value: `${width}x${height}` } : null,
    size ? { label: 'Size', value: size } : null,
    media?.duration || browserMetadata?.duration
      ? { label: 'Duration', value: media?.duration || browserMetadata?.duration || '' }
      : null,
    media?.fps ? { label: 'FPS', value: media.fps } : null,
    media?.codec ? { label: 'Codec', value: media.codec } : null,
    media?.bitrate ? { label: 'Bitrate', value: media.bitrate } : null,
    includeSampleRate && media?.sampleRate
      ? { label: 'Sample Rate', value: `${media.sampleRate}Hz` }
      : null,
    mime ? { label: 'MIME', value: mime } : null,
    format ? { label: 'Format', value: format } : null,
  ].filter((item): item is { label: string; value: string } => Boolean(item?.value));
};

function MediaQuickInfo({
  items,
}: {
  items: Array<{
    label: string;
    value: string;
  }>;
}) {
  if (items.length === 0) {
    return null;
  }

  return (
    <div className="flex flex-wrap gap-1.5 rounded-[14px] border border-border/60 bg-background/80 p-2">
      {items.map((item) => (
        <div
          key={`${item.label}:${item.value}`}
          className="inline-flex min-w-0 items-center gap-1 rounded-full border border-border/60 bg-muted/40 px-2 py-1 text-[11px]"
        >
          <span className="shrink-0 text-muted-foreground">{item.label}</span>
          <span className="min-w-0 max-w-[220px] truncate font-mono text-foreground">
            {item.value}
          </span>
        </div>
      ))}
    </div>
  );
}

export function PrimaryPreviewRenderer({ kind, payload, onOpenFile }: PrimaryPreviewRendererProps) {
  const { t } = useTranslation(['common']);
  const data = payload as PrimaryPayload | null;
  const entry = data?.entry;
  const resolvedData = data?.resolvedData;
  const [loadedImageMetadata, setLoadedImageMetadata] = useState<{
    source: string;
    width?: number;
    height?: number;
  } | null>(null);
  const [loadedVideoMetadata, setLoadedVideoMetadata] = useState<{
    source: string;
    width?: number;
    height?: number;
    duration?: string;
  } | null>(null);
  const content = entry?.content_data || '';
  const metadataString = data?.metadata ? JSON.stringify(data.metadata) : entry?.metadata;
  const imageSrc =
    kind === 'image' ? resolvedData?.imageUrl || resolvedData?.base64?.dataUrl || '' : '';
  const videoSrc =
    kind === 'video' ? resolvedData?.videoUrl || resolvedData?.base64?.dataUrl || '' : '';
  const imageBrowserMetadata =
    loadedImageMetadata?.source === imageSrc ? loadedImageMetadata : undefined;
  const videoBrowserMetadata =
    loadedVideoMetadata?.source === videoSrc ? loadedVideoMetadata : undefined;
  const imageInfoItems = buildMediaInfoItems({
    resolvedData,
    browserMetadata: imageBrowserMetadata,
  });
  const videoInfoItems = buildMediaInfoItems({
    resolvedData,
    browserMetadata: videoBrowserMetadata,
  });
  const audioInfoItems = buildMediaInfoItems({
    resolvedData,
    includeSampleRate: true,
  });
  const imageDrag = useExternalMediaDrag({
    enabled: kind === 'image' && Boolean(imageSrc) && !entry?.file_path,
    kind: 'image',
    sourceUrl: imageSrc || undefined,
    filePath: entry?.file_path || undefined,
    fileName: resolvedData?.fileName,
    mimeType: resolvedData?.mime,
  });
  const videoDrag = useExternalMediaDrag({
    enabled: kind === 'video' && Boolean(videoSrc),
    kind: 'video',
    sourceUrl: videoSrc || undefined,
    filePath: entry?.file_path || undefined,
    fileName: resolvedData?.fileName,
    mimeType: resolvedData?.mime,
  });

  if (!entry) {
    return null;
  }

  if (kind === 'image') {
    if (entry.file_path && resolvedData?.imageUrl) {
      return (
        <ImagePreview
          imageUrl={resolvedData.imageUrl}
          filePath={entry.file_path}
          metadata={data?.metadata?.image_metadata}
          onOpenWithSystem={onOpenFile}
        />
      );
    }

    if (imageSrc) {
      return (
        <div className="space-y-2">
          <MediaQuickInfo items={imageInfoItems} />
          <div className="flex min-h-[300px] items-center justify-center overflow-hidden rounded-[16px] border border-border/70 bg-background/70 p-2.5 min-[1200px]:min-h-[380px] min-[1200px]:p-3">
            <img
              src={imageSrc}
              alt="preview"
              className="max-h-[68vh] max-w-full cursor-grab rounded-xl object-contain active:cursor-grabbing"
              draggable={imageDrag.draggable}
              onDragStart={imageDrag.onDragStart}
              onLoad={(event) => {
                setLoadedImageMetadata({
                  source: imageSrc,
                  width: event.currentTarget.naturalWidth || undefined,
                  height: event.currentTarget.naturalHeight || undefined,
                });
              }}
              title={t('detail.dragToApp', { defaultValue: '拖拽到其他应用发送' })}
            />
          </div>
        </div>
      );
    }
  }

  if (kind === 'video') {
    return (
      <Card>
        <CardHeader className="pb-2">
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <Video className="h-4 w-4" />
              <Badge variant="secondary">
                {t('detail.contentTypes.video', { defaultValue: 'Video' })}
              </Badge>
            </div>
            <MediaQuickInfo items={videoInfoItems} />
          </div>
        </CardHeader>
        <CardContent>
          {videoSrc ? (
            <video
              src={videoSrc}
              controls
              draggable={videoDrag.draggable}
              onDragStart={videoDrag.onDragStart}
              onLoadedMetadata={(event) => {
                setLoadedVideoMetadata({
                  source: videoSrc,
                  width: event.currentTarget.videoWidth || undefined,
                  height: event.currentTarget.videoHeight || undefined,
                  duration: formatDurationFromSeconds(event.currentTarget.duration),
                });
              }}
              title={t('detail.dragToApp', { defaultValue: '拖拽到其他应用发送' })}
              className="max-h-[65vh] w-full cursor-grab rounded-xl border active:cursor-grabbing"
            />
          ) : (
            <div className="py-8 text-sm text-muted-foreground">{t('detail.unknown')}</div>
          )}
        </CardContent>
      </Card>
    );
  }

  if (kind === 'audio') {
    const src = resolvedData?.audioUrl || resolvedData?.base64?.dataUrl;
    return (
      <Card>
        <CardHeader className="pb-2">
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <Music className="h-4 w-4" />
              <Badge variant="secondary">
                {t('detail.contentTypes.audio', { defaultValue: 'Audio' })}
              </Badge>
            </div>
            <MediaQuickInfo items={audioInfoItems} />
          </div>
        </CardHeader>
        <CardContent>{src ? <audio src={src} controls className="w-full" /> : null}</CardContent>
      </Card>
    );
  }

  if (kind === 'json') {
    const textContent =
      resolvedData?.jsonContent !== undefined
        ? getJsonText(resolvedData.jsonContent)
        : resolvedData?.textContent || content;
    return <JsonRenderer content={textContent} />;
  }

  if (kind === 'code') {
    return (
      <UnifiedTextRenderer
        content={resolvedData?.textContent || content}
        contentSubType={data?.subType === 'command' ? 'command' : 'code'}
        metadata={metadataString}
        sessionKey={
          data?.subType === 'command' || data?.subType === 'code' ? data?.sessionKey : undefined
        }
        onContentChange={
          data?.subType === 'command' || data?.subType === 'code'
            ? data?.onContentChange
            : undefined
        }
      />
    );
  }

  if (kind === 'markdown') {
    return (
      <UnifiedTextRenderer
        content={resolvedData?.textContent || content}
        contentSubType="markdown"
        metadata={metadataString}
      />
    );
  }

  if (kind === 'base64_text') {
    return (
      <UnifiedTextRenderer
        content={resolvedData?.base64?.textPreview || resolvedData?.textContent || content}
        contentSubType="plain_text"
      />
    );
  }

  if (kind === 'base64_binary') {
    return (
      <Card>
        <CardContent className="space-y-3 p-4">
          <div className="flex items-center gap-2 text-sm font-medium">
            <FileCode2 className="h-4 w-4" />
            <span>{t('detail.contentTypes.base64', { defaultValue: 'Base64' })}</span>
          </div>
          <div className="rounded-lg bg-muted p-3 text-xs">
            <div>MIME: {resolvedData?.base64?.mime || t('detail.unknown')}</div>
            <div>Size: {resolvedData?.base64?.sizeBytes || 0} bytes</div>
          </div>
        </CardContent>
      </Card>
    );
  }

  if (kind === 'url_card') {
    return (
      <UrlCardRenderer
        raw={content}
        parts={data?.metadata?.url_parts ?? null}
        preview={resolvedData?.url ?? null}
      />
    );
  }

  if (kind === 'file_card') {
    return (
      <div className="flex min-h-[280px] flex-col items-center justify-center rounded-[16px] border border-border/70 bg-background/70 p-5 text-center">
        <div className="mb-3 flex h-12 w-12 items-center justify-center rounded-2xl border border-primary/15 bg-primary/10 text-primary">
          <File className="h-7 w-7" />
        </div>
        <p className="max-w-xl break-all text-sm text-foreground">
          {entry.file_path || content || 'Unknown file'}
        </p>
      </div>
    );
  }

  if (kind === 'color_card') {
    return <ColorRenderer content={content} metadata={metadataString} />;
  }
  if (kind === 'ip_card') {
    return <IpRenderer content={content} />;
  }
  if (kind === 'email_card') {
    return <EmailRenderer content={content} />;
  }
  if (kind === 'timestamp_card') {
    return <TimeRenderer content={content} metadata={metadataString} />;
  }

  return (
    <UnifiedTextRenderer
      content={resolvedData?.textContent || content}
      contentSubType={data?.subType || 'plain_text'}
      metadata={metadataString}
    />
  );
}
