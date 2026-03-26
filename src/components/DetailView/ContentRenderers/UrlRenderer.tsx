import { useState, useEffect } from 'react';
import { Copy, ExternalLink, Globe, FileText, Image, Video, Music } from 'lucide-react';
import queryString from 'query-string';
import { useClipboardStore } from '../../../stores/clipboardStore';
import { UrlParts, ContentSubType } from '../../../types/clipboard';
import { Button } from '../../ui/button';
import { Badge } from '../../ui/badge';
import { Card, CardContent, CardHeader } from '../../ui/card';
import { UnifiedTextRenderer } from './UnifiedTextRenderer';

interface UrlRendererProps {
  content: string;
  metadata?: string | null;
}

interface MediaMetadata {
  bitrate?: string;
  codec?: string;
  duration?: string;
  format?: string;
  fps?: number | string;
  height?: number | string;
  sample_rate?: number | string;
  size?: string;
  width?: number | string;
}

const KNOWN_FILE_EXTENSIONS = new Set([
  '7z',
  'aac',
  'avi',
  'bat',
  'bmp',
  'bz2',
  'c',
  'conf',
  'cpp',
  'css',
  'csv',
  'doc',
  'docx',
  'flac',
  'flv',
  'gif',
  'go',
  'gz',
  'h',
  'hpp',
  'htm',
  'html',
  'ico',
  'ini',
  'java',
  'jpeg',
  'jpg',
  'js',
  'json',
  'jsx',
  'log',
  'm4a',
  'md',
  'mkv',
  'mov',
  'mp3',
  'mp4',
  'ogg',
  'pdf',
  'php',
  'png',
  'ppt',
  'pptx',
  'py',
  'rar',
  'rb',
  'rs',
  'sh',
  'sql',
  'svg',
  'tar',
  'tif',
  'tiff',
  'toml',
  'ts',
  'tsx',
  'txt',
  'wav',
  'webm',
  'webp',
  'xls',
  'xlsx',
  'xml',
  'xz',
  'yaml',
  'yml',
  'zip',
]);

const COMMON_BARE_DOMAIN_TLDS = new Set([
  'ai',
  'app',
  'au',
  'biz',
  'ca',
  'cc',
  'ch',
  'cn',
  'co',
  'com',
  'de',
  'dev',
  'es',
  'fr',
  'in',
  'info',
  'io',
  'it',
  'jp',
  'kr',
  'me',
  'net',
  'nl',
  'no',
  'online',
  'org',
  'ru',
  'se',
  'sh',
  'site',
  'store',
  'tech',
  'tv',
  'uk',
  'us',
  'xyz',
]);

const normalizeUrlContent = (value: string): string | null => {
  const trimmed = value.trim();
  if (!trimmed || /\s/.test(trimmed) || trimmed.includes('@')) {
    return null;
  }

  try {
    const parsed = new URL(trimmed);
    if (
      (parsed.protocol === 'http:' || parsed.protocol === 'https:' || parsed.protocol === 'ftp:') &&
      parsed.host
    ) {
      return parsed.toString();
    }
    return null;
  } catch {
    // fall through to bare-domain heuristics
  }

  const boundaryCandidates = ['/', '?', '#']
    .map((separator) => trimmed.indexOf(separator))
    .filter((index) => index >= 0);
  const boundary = boundaryCandidates.length > 0 ? Math.min(...boundaryCandidates) : trimmed.length;
  const hostAndPort = trimmed.slice(0, boundary);

  if (!hostAndPort) {
    return null;
  }

  let host = hostAndPort;
  let hasExtraParts = boundary < trimmed.length;
  const lastColon = hostAndPort.lastIndexOf(':');
  if (lastColon > 0) {
    const port = hostAndPort.slice(lastColon + 1);
    if (/^\d+$/.test(port)) {
      host = hostAndPort.slice(0, lastColon);
      hasExtraParts = true;
    }
  }

  if (!/^(?:[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]{2,24}$/.test(host)) {
    return null;
  }

  const labels = host.split('.');
  const tld = labels[labels.length - 1]?.toLowerCase();
  if (!tld || KNOWN_FILE_EXTENSIONS.has(tld)) {
    return null;
  }

  if (
    !hasExtraParts &&
    !host.startsWith('www.') &&
    labels.length === 2 &&
    !COMMON_BARE_DOMAIN_TLDS.has(tld)
  ) {
    return null;
  }

  return `https://${trimmed}`;
};

const getPreviewTypeForContent = (value: string): 'none' | 'image' | 'video' | 'audio' | 'text' => {
  if (value.match(/\.(jpg|jpeg|png|gif|webp|svg|bmp|ico)(\?|$)/i)) {
    return 'image';
  }

  if (value.match(/\.(mp4|webm|ogg|avi|mov|mkv|flv)(\?|$)/i)) {
    return 'video';
  }

  if (value.match(/\.(mp3|wav|flac|aac|ogg|m4a)(\?|$)/i)) {
    return 'audio';
  }

  if (
    value.match(
      /\.(json|xml|html|htm|css|js|ts|jsx|tsx|py|java|cpp|c|h|php|rb|go|rs|sql|md|txt|log|csv|yaml|yml|toml|ini|conf|sh|bat)(\?|$)/i
    ) ||
    value.includes('/api/')
  ) {
    return 'text';
  }

  return 'none';
};

export function UrlRenderer({ content, metadata }: UrlRendererProps) {
  const { copyToClipboard, fetchUrlContent, checkFFprobeAvailable, extractMediaMetadata } =
    useClipboardStore();
  const normalizedUrl = normalizeUrlContent(content);
  const fetchableUrl =
    normalizedUrl && (normalizedUrl.startsWith('http://') || normalizedUrl.startsWith('https://'))
      ? normalizedUrl
      : null;
  const [urlParts, setUrlParts] = useState<UrlParts | null>(null);
  const [previewType, setPreviewType] = useState<'none' | 'image' | 'video' | 'audio' | 'text'>(
    'none'
  );
  const [textContent, setTextContent] = useState<string>('');
  const [textContentType, setTextContentType] = useState<ContentSubType>('plain_text');
  const [mediaMetadata, setMediaMetadata] = useState<MediaMetadata | null>(null);
  const [isLoadingText, setIsLoadingText] = useState(false);
  const mediaPreviewUrl = normalizedUrl ?? content;

  const getContentTypeFromUrl = (url: string): ContentSubType => {
    if (url.match(/\.(json)(\?|$)/i) || url.includes('/api/')) {
      return 'json';
    } else if (url.match(/\.(html|htm)(\?|$)/i)) {
      return 'code'; // HTML will be detected by UnifiedTextRenderer
    } else if (url.match(/\.(css)(\?|$)/i)) {
      return 'code';
    } else if (url.match(/\.(js|jsx|ts|tsx)(\?|$)/i)) {
      return 'code';
    } else if (url.match(/\.(py|java|cpp|c|h|php|rb|go|rs)(\?|$)/i)) {
      return 'code';
    } else if (url.match(/\.(md)(\?|$)/i)) {
      return 'markdown';
    } else if (url.match(/\.(sh|bat)(\?|$)/i)) {
      return 'command';
    }
    return 'plain_text';
  };

  useEffect(() => {
    let isActive = true;
    let nextUrlParts: UrlParts | null = null;

    if (metadata) {
      try {
        const parsed = JSON.parse(metadata);
        nextUrlParts = parsed.url_parts ?? null;
      } catch (e) {
        console.warn('Metadata parsing failed:', e);
      }
    }

    if (!nextUrlParts && normalizedUrl) {
      try {
        const url = new URL(normalizedUrl);
        const parsedUrl = queryString.parseUrl(normalizedUrl);
        const queryParams = Object.entries(parsedUrl.query || {}).map(
          ([k, v]) => [k, Array.isArray(v) ? v.join(',') : String(v)] as [string, string]
        );

        nextUrlParts = {
          protocol: url.protocol.replace(':', ''),
          host: url.host,
          path: url.pathname,
          query_params: queryParams,
        };
      } catch (e) {
        console.error('URL解析失败:', e);
      }
    }

    setUrlParts(nextUrlParts);
    const nextPreviewType = getPreviewTypeForContent(content);
    setPreviewType(nextPreviewType);
    setTextContent('');
    setTextContentType('plain_text');
    setIsLoadingText(false);

    if (nextPreviewType !== 'text' || !fetchableUrl) {
      return () => {
        isActive = false;
      };
    }

    setIsLoadingText(true);

    void (async () => {
      try {
        // 使用Tauri命令获取URL内容，绕过浏览器CORS限制
        const data = await fetchUrlContent(fetchableUrl);
        if (!isActive) {
          return;
        }

        // 根据URL扩展名确定内容类型
        const contentType = getContentTypeFromUrl(fetchableUrl);

        // 如果是JSON内容，尝试格式化
        let processedContent = data;
        if (contentType === 'json') {
          try {
            const parsed = JSON.parse(data);
            processedContent = JSON.stringify(parsed, null, 2);
          } catch (jsonError) {
            console.log('JSON 解析失败，显示原始内容:', jsonError);
            // 如果JSON解析失败，就使用原始内容
          }
        }

        if (!isActive) {
          return;
        }

        setTextContent(processedContent);
        setTextContentType(contentType);
      } catch (e) {
        if (!isActive) {
          return;
        }

        console.error('获取URL内容失败:', e);
        // 如果获取失败，显示提示信息
        setTextContent(`// 无法获取URL内容\n// 错误信息: ${String(e)}`);
        setTextContentType('plain_text');
      } finally {
        if (isActive) {
          setIsLoadingText(false);
        }
      }
    })();

    return () => {
      isActive = false;
    };
  }, [content, fetchUrlContent, metadata, fetchableUrl, normalizedUrl]);

  // 在组件加载时检查 FFprobe 可用性并自动获取媒体元数据
  useEffect(() => {
    let isActive = true;

    setMediaMetadata(null);

    const initialize = async () => {
      if (!normalizedUrl || !['image', 'video', 'audio'].includes(previewType)) {
        return;
      }

      const available = await checkFFprobeAvailable();
      if (!isActive || !available) {
        return;
      }

      // 如果是媒体文件且 FFprobe 可用，自动获取元数据
      if (previewType === 'image' || previewType === 'video' || previewType === 'audio') {
        try {
          const nextMetadata = await extractMediaMetadata(normalizedUrl);
          if (!isActive) {
            return;
          }

          setMediaMetadata(nextMetadata);
        } catch (error) {
          if (!isActive) {
            return;
          }

          console.error('自动获取媒体元数据失败:', error);
        }
      }
    };

    void initialize();

    return () => {
      isActive = false;
    };
  }, [checkFFprobeAvailable, extractMediaMetadata, normalizedUrl, previewType]);

  const handleCopy = async (text: string) => {
    await copyToClipboard(text);
  };

  const handleOpenUrl = () => {
    if (normalizedUrl) {
      window.open(normalizedUrl, '_blank');
    }
  };

  if (!normalizedUrl && !urlParts) {
    return <UnifiedTextRenderer content={content} contentSubType="plain_text" />;
  }

  return (
    <div id="url-renderer" className="space-y-4">
      <Card id="url-renderer-info">
        <CardHeader id="url-renderer-header" className="pb-3">
          <div id="url-renderer-toolbar" className="flex items-center justify-between">
            <div id="url-renderer-badges" className="flex items-center gap-2">
              <Globe className="w-4 h-4" />
              <Badge variant="secondary">URL链接</Badge>
            </div>
            <div id="url-renderer-actions" className="flex gap-2">
              <Button
                id="url-renderer-copy-btn"
                onClick={() => handleCopy(content)}
                size="sm"
                variant="outline"
              >
                <Copy className="w-4 h-4 mr-2" />
                复制URL
              </Button>
              <Button
                id="url-renderer-open-btn"
                onClick={handleOpenUrl}
                size="sm"
                variant="outline"
                disabled={!normalizedUrl}
              >
                <ExternalLink className="w-4 h-4 mr-2" />
                打开链接
              </Button>
            </div>
          </div>
        </CardHeader>

        <CardContent className="space-y-3">
          <div>
            <code className="block p-2 bg-muted rounded text-xs font-mono break-all">
              {content}
            </code>
          </div>

          {urlParts && (
            <div className="space-y-2">
              <div className="flex flex-wrap gap-2 text-xs">
                <span className="inline-flex items-center gap-1 px-2 py-1 bg-muted rounded">
                  <span className="text-muted-foreground">协议:</span>
                  <code className="font-mono">{urlParts.protocol}</code>
                </span>
                <span className="inline-flex items-center gap-1 px-2 py-1 bg-muted rounded">
                  <span className="text-muted-foreground">主机:</span>
                  <code className="font-mono">{urlParts.host}</code>
                </span>
                {urlParts.path && urlParts.path !== '/' && (
                  <span className="inline-flex items-center gap-1 px-2 py-1 bg-muted rounded">
                    <span className="text-muted-foreground">路径:</span>
                    <code className="font-mono">{urlParts.path}</code>
                  </span>
                )}
              </div>

              {urlParts.query_params.length > 0 && (
                <details className="group">
                  <summary className="cursor-pointer text-xs text-muted-foreground hover:text-foreground">
                    {urlParts.query_params.length} 个查询参数
                  </summary>
                  <div className="mt-2 space-y-1">
                    {urlParts.query_params.map(([key, value], index) => (
                      <div
                        key={index}
                        className="flex items-center gap-1 p-1 bg-muted rounded text-xs"
                      >
                        <code className="font-medium text-primary">{key}</code>
                        <span className="text-muted-foreground">=</span>
                        <code className="flex-1 break-all text-muted-foreground">{value}</code>
                        <Button
                          onClick={() => handleCopy(value)}
                          size="sm"
                          variant="ghost"
                          className="h-5 w-5 p-0"
                        >
                          <Copy className="w-3 h-3" />
                        </Button>
                      </div>
                    ))}
                  </div>
                </details>
              )}
            </div>
          )}
        </CardContent>
      </Card>

      {previewType === 'image' && (
        <Card>
          <CardHeader className="pb-2">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium flex items-center gap-2">
                <Image className="w-4 h-4" />
                图片预览
              </span>
              {mediaMetadata && (
                <div className="flex items-center gap-2 text-xs text-muted-foreground">
                  {mediaMetadata.width && mediaMetadata.height && (
                    <span>
                      {mediaMetadata.width}x{mediaMetadata.height}
                    </span>
                  )}
                  {mediaMetadata.format && <span>• {mediaMetadata.format}</span>}
                  {mediaMetadata.size && <span>• {mediaMetadata.size}</span>}
                </div>
              )}
            </div>
          </CardHeader>
          <CardContent>
            <img
              src={mediaPreviewUrl}
              alt="预览"
              className="max-w-full h-auto rounded border"
              onError={(e) => {
                e.currentTarget.style.display = 'none';
              }}
            />
          </CardContent>
        </Card>
      )}

      {previewType === 'video' && (
        <Card>
          <CardHeader className="pb-2">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium flex items-center gap-2">
                <Video className="w-4 h-4" />
                视频预览
              </span>
              {mediaMetadata && (
                <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
                  {mediaMetadata.width && mediaMetadata.height && (
                    <span>
                      {mediaMetadata.width}x{mediaMetadata.height}
                    </span>
                  )}
                  {mediaMetadata.fps && <span>• {mediaMetadata.fps}fps</span>}
                  {mediaMetadata.duration && <span>• {mediaMetadata.duration}</span>}
                  {mediaMetadata.codec && <span>• {mediaMetadata.codec}</span>}
                  {mediaMetadata.bitrate && <span>• {mediaMetadata.bitrate}</span>}
                </div>
              )}
            </div>
          </CardHeader>
          <CardContent>
            <video src={mediaPreviewUrl} controls className="max-w-full h-auto rounded border" />
          </CardContent>
        </Card>
      )}

      {previewType === 'audio' && (
        <Card>
          <CardHeader className="pb-2">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium flex items-center gap-2">
                <Music className="w-4 h-4" />
                音频预览
              </span>
              {mediaMetadata && (
                <div className="flex items-center gap-2 text-xs text-muted-foreground">
                  {mediaMetadata.duration && <span>{mediaMetadata.duration}</span>}
                  {mediaMetadata.bitrate && <span>• {mediaMetadata.bitrate}</span>}
                  {mediaMetadata.sample_rate && <span>• {mediaMetadata.sample_rate}Hz</span>}
                  {mediaMetadata.codec && <span>• {mediaMetadata.codec}</span>}
                </div>
              )}
            </div>
          </CardHeader>
          <CardContent>
            <audio src={mediaPreviewUrl} controls className="w-full" />
          </CardContent>
        </Card>
      )}

      {previewType === 'text' && (
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center gap-2">
              <FileText className="w-4 h-4" />
              <span className="text-sm font-medium">内容预览</span>
              {isLoadingText && <span className="text-xs text-muted-foreground">加载中...</span>}
            </div>
          </CardHeader>
          <CardContent className="p-0">
            {textContent && (
              <div className="h-[500px]">
                <UnifiedTextRenderer content={textContent} contentSubType={textContentType} />
              </div>
            )}
          </CardContent>
        </Card>
      )}
    </div>
  );
}
