import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Badge } from '../ui/badge';
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

const parseMetadata = (metadataString?: string | null): ContentMetadata | null => {
  if (!metadataString) return null;
  try {
    return JSON.parse(metadataString) as ContentMetadata;
  } catch {
    return null;
  }
};

export function DetailView() {
  const { t } = useTranslation(['common']);
  const { selectedEntry, getImageUrl, openFileWithSystem } = useClipboardStore();
  const [imageUrl, setImageUrl] = useState<string>('');
  const [contentSubType, setContentSubType] = useState<ContentSubType>('plain_text');

  useEffect(() => {
    const loadImage = async () => {
      if (selectedEntry?.file_path && selectedEntry.content_type.toLowerCase().includes('image')) {
        try {
          console.log('[DetailView] 开始加载图片:', selectedEntry.file_path);
          const url = await getImageUrl(selectedEntry.file_path);
          console.log('[DetailView] 图片URL获取成功，长度:', url.length);
          setImageUrl(url);
        } catch (error) {
          console.error('[DetailView] 图片加载失败:', error);
          console.error('[DetailView] 文件路径:', selectedEntry.file_path);
          setImageUrl('');
        }
      } else {
        setImageUrl('');
      }
    };

    loadImage();

    // 解析内容子类型
    if (selectedEntry?.content_subtype) {
      console.log('[DetailView] 收到的子类型:', selectedEntry.content_subtype);
      // content_subtype是直接的字符串，不需要JSON.parse
      setContentSubType(selectedEntry.content_subtype as ContentSubType);
    } else {
      console.log('[DetailView] 没有子类型信息，使用默认值');
      setContentSubType('plain_text');
    }
  }, [selectedEntry, getImageUrl]);

  if (!selectedEntry) {
    return (
      <Card id="detail-view-empty" className="flex-1 flex flex-col">
        <CardContent
          id="detail-view-empty-content"
          className="flex-1 flex items-center justify-center p-8"
        >
          <div id="detail-view-empty-message" className="text-center text-muted-foreground">
            <p>{t('detail.selectItem')}</p>
          </div>
        </CardContent>
      </Card>
    );
  }

  const formatDate = (timestamp: number) => {
    const date = new Date(timestamp);
    return date.toLocaleString(undefined, {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  };

  const getContentType = () => {
    const type = selectedEntry.content_type.toLowerCase();

    // 如果是文本类型，显示具体的子类型
    if (type.includes('text') || type.includes('string')) {
      return t(`detail.contentTypes.${contentSubType}`) || t('detail.contentTypes.text');
    }

    if (type.includes('image')) return t('detail.contentTypes.image');
    if (type.includes('file')) return t('detail.contentTypes.file');
    return t('detail.unknown');
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

  const renderContent = () => {
    if (!selectedEntry) return null;

    // 图片类型
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
      } else {
        return (
          <div className="flex flex-col items-center justify-center p-8 text-center">
            <div className="w-8 h-8 border-2 border-primary border-t-transparent rounded-full animate-spin mb-4" />
            <p className="text-muted-foreground">{t('detail.loading')}</p>
            {selectedEntry.file_path && (
              <p className="text-xs text-muted-foreground mt-2 break-all">
                {selectedEntry.file_path}
              </p>
            )}
          </div>
        );
      }
    }

    // 文件类型
    if (selectedEntry.content_type.toLowerCase().includes('file')) {
      return (
        <div className="flex flex-col items-center justify-center p-8 text-center">
          <div className="text-4xl mb-4">📁</div>
          <p className="text-sm text-muted-foreground break-all">
            {selectedEntry.file_path || selectedEntry.content_data}
          </p>
        </div>
      );
    }

    // 文本类型 - 根据子类型选择不同的渲染器
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

  return (
    <Card id="detail-view" className="flex-1 flex flex-col overflow-hidden">
      <CardHeader id="detail-view-header" className="pb-3">
        <CardTitle id="detail-view-title" className="text-lg">
          {t('detail.title')}
        </CardTitle>
        <div id="detail-view-metadata" className="grid grid-cols-2 gap-3 mt-3 text-sm">
          <div id="detail-view-type" className="flex items-center gap-2">
            <span className="text-muted-foreground">{t('detail.type')}</span>
            <Badge variant="secondary" className="text-xs">
              {getContentType()}
            </Badge>
          </div>
          <div id="detail-view-source" className="flex items-center gap-2">
            <span className="text-muted-foreground">{t('detail.source')}</span>
            <span className="text-foreground font-medium">
              {selectedEntry.source_app || t('detail.unknown')}
            </span>
          </div>
          <div id="detail-view-time" className="flex items-center gap-2">
            <span className="text-muted-foreground">{t('detail.time')}</span>
            <span className="text-foreground font-mono text-xs">
              {formatDate(selectedEntry.created_at)}
            </span>
          </div>
          <div id="detail-view-count" className="flex items-center gap-2">
            <span className="text-muted-foreground">{t('detail.copyCount')}</span>
            <Badge variant="outline" className="text-xs">
              {selectedEntry.copy_count}
            </Badge>
          </div>
        </div>
      </CardHeader>

      <CardContent id="detail-view-content" className="flex-1 overflow-hidden p-0">
        <div id="detail-view-scroll" className="h-full overflow-y-auto">
          <div id="detail-view-content-wrapper" className="p-6 h-full flex flex-col">
            {renderContent()}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
