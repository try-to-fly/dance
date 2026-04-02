import { File, FileCode2, Music, Video } from 'lucide-react';
import { useTranslation } from 'react-i18next';
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

export function PrimaryPreviewRenderer({ kind, payload, onOpenFile }: PrimaryPreviewRendererProps) {
  const { t } = useTranslation(['common']);
  const data = payload as PrimaryPayload | null;
  const entry = data?.entry;
  const resolvedData = data?.resolvedData;
  const content = entry?.content_data || '';
  const metadataString = data?.metadata ? JSON.stringify(data.metadata) : entry?.metadata;

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

    const src = resolvedData?.imageUrl || resolvedData?.base64?.dataUrl || '';
    if (src) {
      return (
        <div className="flex min-h-[300px] items-center justify-center overflow-hidden rounded-[16px] border border-border/70 bg-background/70 p-2.5 min-[1200px]:min-h-[380px] min-[1200px]:p-3">
          <img
            src={src}
            alt="preview"
            className="max-h-[68vh] max-w-full rounded-xl object-contain"
          />
        </div>
      );
    }
  }

  if (kind === 'video') {
    const src = resolvedData?.videoUrl || resolvedData?.base64?.dataUrl;
    return (
      <Card>
        <CardHeader className="pb-2">
          <div className="flex items-center gap-2">
            <Video className="h-4 w-4" />
            <Badge variant="secondary">
              {t('detail.contentTypes.video', { defaultValue: 'Video' })}
            </Badge>
          </div>
        </CardHeader>
        <CardContent>
          {src ? (
            <video src={src} controls className="max-h-[65vh] w-full rounded-xl border" />
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
          <div className="flex items-center gap-2">
            <Music className="h-4 w-4" />
            <Badge variant="secondary">
              {t('detail.contentTypes.audio', { defaultValue: 'Audio' })}
            </Badge>
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
    return <UrlCardRenderer raw={content} parts={data?.metadata?.url_parts ?? null} />;
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
