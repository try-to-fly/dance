import React, { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { Copy, Download, Maximize2 } from 'lucide-react';
import { cn } from '../../lib/utils';
import { useExternalMediaDrag } from '../../hooks/useExternalMediaDrag';
import { Button } from '../ui/button';
import { ScrollArea } from '../ui/scroll-area';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select';

interface ImagePreviewProps {
  imageUrl: string;
  filePath: string;
  metadata?: {
    width: number;
    height: number;
    file_size: number;
    format?: string;
  };
  onOpenWithSystem?: () => void;
}

const formatFileSize = (bytes: number): string => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
};

export const ImagePreview: React.FC<ImagePreviewProps> = ({
  imageUrl,
  filePath,
  metadata,
  onOpenWithSystem,
}) => {
  const { t } = useTranslation(['common']);
  const [selectedFormat, setSelectedFormat] = useState<string>('png');
  const [selectedScale, setSelectedScale] = useState<number>(1.0);
  const [previewUrl, setPreviewUrl] = useState<string>(imageUrl);
  const [isConverting, setIsConverting] = useState(false);
  const [convertedMetadata, setConvertedMetadata] = useState<{
    size: number;
    width: number;
    height: number;
  } | null>(null);
  const [showOriginal, setShowOriginal] = useState(true);

  useEffect(() => {
    setPreviewUrl(imageUrl);
    setConvertedMetadata(null);
    setShowOriginal(true);
  }, [imageUrl]);

  const activeImageSrc = showOriginal ? imageUrl : previewUrl;
  const dragMedia = useExternalMediaDrag({
    enabled: Boolean(activeImageSrc),
    kind: 'image',
    sourceUrl: activeImageSrc,
    filePath: showOriginal ? filePath : undefined,
    fileName: undefined,
    mimeType: undefined,
  });

  const handleConvert = async () => {
    setIsConverting(true);
    try {
      const convertedData = await invoke<string>('convert_and_scale_image', {
        filePath,
        format: selectedFormat,
        scale: selectedScale,
        skipRecording: true,
      });

      setPreviewUrl(convertedData);
      setShowOriginal(false);

      const base64Part = convertedData.split(',')[1];
      if (base64Part && metadata) {
        const binarySize = atob(base64Part).length;
        const newWidth = Math.round(metadata.width * selectedScale);
        const newHeight = Math.round(metadata.height * selectedScale);

        setConvertedMetadata({
          size: binarySize,
          width: newWidth,
          height: newHeight,
        });
      }
    } catch (error) {
      console.error('Failed to convert image:', error);
      alert(t('imagePreview.convertError', { error: String(error) }));
    } finally {
      setIsConverting(false);
    }
  };

  const handleCopyConverted = async () => {
    try {
      await invoke('copy_converted_image', {
        base64Data: previewUrl,
        skipRecording: true,
      });

      const toast = document.createElement('div');
      toast.className = 'toast-notification';
      toast.textContent = t('imagePreview.copySuccess');
      document.body.appendChild(toast);

      setTimeout(() => {
        toast.remove();
      }, 2000);
    } catch (error) {
      console.error('Failed to copy converted image:', error);
      alert(t('imagePreview.copyError', { error: String(error) }));
    }
  };

  const handleDownload = () => {
    const link = document.createElement('a');
    link.href = previewUrl;
    const extension = previewUrl.includes('jpeg')
      ? 'jpg'
      : previewUrl.includes('webp')
        ? 'webp'
        : 'png';
    const scaleSuffix = selectedScale !== 1.0 ? `_${Math.round(selectedScale * 100)}` : '';
    link.download = `image${scaleSuffix}.${extension}`;
    link.click();
  };

  const previewVariants = [
    metadata && {
      key: 'original',
      active: showOriginal,
      label: t('imagePreview.original'),
      summary: `${metadata.width}×${metadata.height} · ${formatFileSize(metadata.file_size)}`,
      onClick: () => {
        setShowOriginal(true);
      },
    },
    convertedMetadata && {
      key: 'converted',
      active: !showOriginal,
      label: t('imagePreview.converted'),
      summary: `${convertedMetadata.width}×${convertedMetadata.height} · ${formatFileSize(
        convertedMetadata.size
      )}`,
      onClick: () => {
        setShowOriginal(false);
      },
    },
  ].filter(Boolean) as Array<{
    key: string;
    active: boolean;
    label: string;
    summary: string;
    onClick: () => void;
  }>;

  return (
    <div id="image-preview" className="flex min-h-0 flex-col gap-2">
      <div
        id="image-preview-controls"
        className="rounded-2xl border border-border/60 bg-card/45 px-2.5 py-2 shadow-sm backdrop-blur-sm"
      >
        <div id="image-preview-toolbar" className="flex flex-wrap items-center gap-1.5">
          <div className="flex items-center gap-1.5 rounded-full border border-border/60 bg-background/75 px-2 py-1">
            <span className="text-[11px] font-medium text-muted-foreground">
              {t('imagePreview.format')}
            </span>
            <Select value={selectedFormat} onValueChange={setSelectedFormat}>
              <SelectTrigger className="h-7 w-[86px] rounded-full border-0 bg-transparent px-2 text-xs shadow-none ring-0 focus:ring-0 focus:ring-offset-0">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="png">PNG</SelectItem>
                <SelectItem value="jpeg">JPEG</SelectItem>
                <SelectItem value="webp">WebP</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="flex items-center gap-1.5 rounded-full border border-border/60 bg-background/75 px-2 py-1">
            <span className="text-[11px] font-medium text-muted-foreground">
              {t('imagePreview.scale')}
            </span>
            <Select
              value={selectedScale.toString()}
              onValueChange={(value) => setSelectedScale(parseFloat(value))}
            >
              <SelectTrigger className="h-7 w-[78px] rounded-full border-0 bg-transparent px-2 text-xs shadow-none ring-0 focus:ring-0 focus:ring-offset-0">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="0.3">30%</SelectItem>
                <SelectItem value="0.5">50%</SelectItem>
                <SelectItem value="0.8">80%</SelectItem>
                <SelectItem value="1.0">100%</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <Button
            onClick={handleConvert}
            disabled={isConverting}
            size="sm"
            variant="secondary"
            className="h-8 rounded-full px-3 text-xs"
          >
            {isConverting ? t('imagePreview.converting') : t('imagePreview.convert')}
          </Button>

          <div className="ml-auto flex flex-wrap items-center gap-1">
            {convertedMetadata && (
              <>
                <Button
                  onClick={handleCopyConverted}
                  size="sm"
                  variant="outline"
                  className="h-8 rounded-full border-border/60 bg-background/75 px-2.5 text-xs"
                  aria-label={t('imagePreview.copy')}
                  title={t('imagePreview.copy')}
                >
                  <Copy className="h-3.5 w-3.5" />
                  <span className="hidden min-[1200px]:inline">{t('imagePreview.copy')}</span>
                </Button>
                <Button
                  onClick={handleDownload}
                  size="icon"
                  variant="outline"
                  className="h-8 w-8 rounded-full border-border/60 bg-background/75"
                  aria-label="Download"
                  title="Download"
                >
                  <Download className="h-3.5 w-3.5" />
                </Button>
              </>
            )}

            {onOpenWithSystem && (
              <Button
                onClick={onOpenWithSystem}
                size="icon"
                variant="outline"
                className="h-8 w-8 rounded-full border-border/60 bg-background/75"
                aria-label="Open in system"
                title="Open in system"
              >
                <Maximize2 className="h-3.5 w-3.5" />
              </Button>
            )}
          </div>
        </div>

        {previewVariants.length > 0 && (
          <div className="mt-2 flex flex-wrap items-center gap-1.5">
            {previewVariants.map((variant) => (
              <button
                type="button"
                key={variant.key}
                onClick={variant.onClick}
                className={cn(
                  'inline-flex items-center gap-2 rounded-full border px-2.5 py-1 text-left transition-colors',
                  variant.active
                    ? 'border-primary/25 bg-primary/10 text-foreground'
                    : 'border-border/60 bg-background/55 text-muted-foreground hover:bg-accent hover:text-accent-foreground'
                )}
              >
                <span
                  className={cn(
                    'rounded-full px-1.5 py-0.5 text-[10px] font-semibold',
                    variant.active
                      ? 'bg-primary/15 text-foreground'
                      : 'bg-muted text-muted-foreground'
                  )}
                >
                  {variant.label}
                </span>
                <span className="text-[11px] font-medium">{variant.summary}</span>
              </button>
            ))}
          </div>
        )}
      </div>

      <div
        id="image-preview-display"
        className="overflow-hidden rounded-2xl border border-border/60 bg-background/80"
      >
        <ScrollArea id="image-preview-scroll" className="max-h-[68vh] min-[1200px]:max-h-[72vh]">
          <div
            id="image-preview-wrapper"
            className="flex min-h-[240px] items-center justify-center p-2 min-[1200px]:min-h-[320px] min-[1200px]:p-3"
          >
            <img
              id="image-preview-img"
              src={activeImageSrc}
              alt={showOriginal ? t('imagePreview.originalAlt') : t('imagePreview.convertedAlt')}
              className="max-h-[66vh] max-w-full rounded-xl border border-border/60 bg-card/60 object-contain transition-colors hover:border-primary/50 min-[1200px]:max-h-[70vh]"
              onClick={onOpenWithSystem}
              draggable={dragMedia.draggable}
              onDragStart={dragMedia.onDragStart}
              title={t('imagePreview.dragToApp', { defaultValue: '拖拽到其他应用发送' })}
            />
          </div>
        </ScrollArea>
      </div>
    </div>
  );
};
