import React, { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { Copy, Download, Maximize2 } from 'lucide-react';
import { Badge } from '../ui/badge';
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
        setPreviewUrl(imageUrl);
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
    <div id="image-preview" className="flex min-h-0 flex-col gap-2.5">
      <div
        id="image-preview-controls"
        className="rounded-[18px] border border-border/70 bg-card/70 px-3 py-3 backdrop-blur-sm"
      >
        <div id="image-preview-toolbar" className="flex flex-wrap items-center gap-2">
          <div className="flex items-center gap-2">
            <span className="text-xs font-medium text-muted-foreground">
              {t('imagePreview.format')}
            </span>
            <Select value={selectedFormat} onValueChange={setSelectedFormat}>
              <SelectTrigger className="h-9 w-[88px] rounded-xl bg-background/80 px-3">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="png">PNG</SelectItem>
                <SelectItem value="jpeg">JPEG</SelectItem>
                <SelectItem value="webp">WebP</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="flex items-center gap-2">
            <span className="text-xs font-medium text-muted-foreground">
              {t('imagePreview.scale')}
            </span>
            <Select
              value={selectedScale.toString()}
              onValueChange={(value) => setSelectedScale(parseFloat(value))}
            >
              <SelectTrigger className="h-9 w-[82px] rounded-xl bg-background/80 px-3">
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
            className="h-9 rounded-xl px-4"
          >
            {isConverting ? t('imagePreview.converting') : t('imagePreview.convert')}
          </Button>

          <div className="ml-auto flex flex-wrap items-center gap-2">
            {convertedMetadata && (
              <>
                <Button
                  onClick={handleCopyConverted}
                  size="sm"
                  variant="outline"
                  className="h-9 rounded-xl px-3"
                >
                  <Copy className="mr-1 h-4 w-4" />
                  {t('imagePreview.copy')}
                </Button>
                <Button
                  onClick={handleDownload}
                  size="sm"
                  variant="outline"
                  className="h-9 rounded-xl px-3"
                >
                  <Download className="h-4 w-4" />
                </Button>
              </>
            )}

            {onOpenWithSystem && (
              <Button
                onClick={onOpenWithSystem}
                size="sm"
                variant="outline"
                className="h-9 rounded-xl px-3"
              >
                <Maximize2 className="h-4 w-4" />
              </Button>
            )}
          </div>
        </div>

        {previewVariants.length > 0 && (
          <div className="mt-2 flex flex-wrap gap-2">
            {previewVariants.map((variant) => (
              <Button
                key={variant.key}
                variant={variant.active ? 'default' : 'outline'}
                size="sm"
                onClick={variant.onClick}
                className="h-9 rounded-full px-3 text-xs"
              >
                <Badge variant="secondary" className="mr-2 rounded-full px-2 py-0.5 text-[10px]">
                  {variant.label}
                </Badge>
                {variant.summary}
              </Button>
            ))}
          </div>
        )}
      </div>

      <div
        id="image-preview-display"
        className="overflow-hidden rounded-[18px] border border-border/70 bg-background/80"
      >
        <ScrollArea id="image-preview-scroll" className="max-h-[68vh] min-[1200px]:max-h-[72vh]">
          <div
            id="image-preview-wrapper"
            className="flex min-h-[280px] items-center justify-center p-2.5 min-[1200px]:min-h-[340px] min-[1200px]:p-3"
          >
            <img
              id="image-preview-img"
              src={showOriginal ? imageUrl : previewUrl}
              alt={showOriginal ? t('imagePreview.originalAlt') : t('imagePreview.convertedAlt')}
              className="max-h-[66vh] max-w-full rounded-xl border border-border/60 bg-card/60 object-contain transition-colors hover:border-primary/50 min-[1200px]:max-h-[70vh]"
              onClick={onOpenWithSystem}
            />
          </div>
        </ScrollArea>
      </div>
    </div>
  );
};
