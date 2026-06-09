import { describe, expect, it } from 'vitest';
import { ClipboardEntry, ResolvedPreviewData } from '../../types/clipboard';
import { buildPreviewDescriptor } from './previewDescriptor';

const labels = {
  unknown: 'Unknown',
  image: 'Image',
  file: 'File',
  text: 'Text',
  base64: 'Base64',
  subtypeLabels: {
    url: 'URL',
  },
};

const baseUrlEntry: ClipboardEntry = {
  id: 'entry-url-media',
  content_hash: 'hash-url-media',
  content_type: 'text/plain',
  content_data: 'https://cdn.example.com/video.mp4',
  source_app: 'Safari',
  created_at: new Date('2026-05-26T10:00:00Z').getTime(),
  copy_count: 1,
  file_path: null,
  is_favorite: false,
  content_subtype: 'url',
  metadata: null,
  app_bundle_id: null,
  analysis: {
    contract_version: 1,
    analysis_version: 1,
    status: 'matched',
    subtype: 'url',
    metadata: {
      kind: 'url',
      data: {
        protocol: 'https',
        host: 'cdn.example.com',
        path: '/video.mp4',
        query_params: [],
      },
    },
    diagnostics: [],
    analyzed_at: new Date('2026-05-26T10:00:00Z').getTime(),
  },
};

const getSectionItems = (resolvedData: ResolvedPreviewData) => {
  const descriptor = buildPreviewDescriptor({
    entry: baseUrlEntry,
    resolvedData,
    labels,
  });
  const mediaSection = descriptor.inspectorSections.find((section) => section.title === 'Media');

  return {
    descriptor,
    items: new Map(mediaSection?.items.map((item) => [item.label, item.value]) ?? []),
  };
};

describe('buildPreviewDescriptor media inspector', () => {
  it('URL 视频预览会展示完整媒体属性', () => {
    const { descriptor, items } = getSectionItems({
      sourceKind: 'remote',
      mime: 'video/mp4',
      extension: 'mp4',
      sizeBytes: 5_242_880,
      videoUrl: 'https://cdn.example.com/video.mp4',
      media: {
        width: 1920,
        height: 1080,
        duration: '1:23',
        fps: '29.97',
        codec: 'h264',
        bitrate: '1200 kbps',
        sizeBytes: 5_242_880,
        size: '5.0 MB',
        format: 'mp4',
      },
      url: {
        finalUrl: 'https://cdn.example.com/video.mp4',
        contentLength: 5_242_880,
        contentType: 'video/mp4',
        previewKind: 'video',
      },
    });

    expect(descriptor.primaryKind).toBe('video');
    expect(items.get('Resolution')).toBe('1920x1080');
    expect(items.get('Size')).toBe('5.0 MB');
    expect(items.get('MIME')).toBe('video/mp4');
    expect(items.get('Format')).toBe('MP4');
    expect(items.get('Duration')).toBe('1:23');
    expect(items.get('FPS')).toBe('29.97');
    expect(items.get('Codec')).toBe('h264');
    expect(items.get('Bitrate')).toBe('1200 kbps');
  });

  it('URL 图片没有 ffprobe 结果时仍展示响应头媒体属性', () => {
    const { descriptor, items } = getSectionItems({
      sourceKind: 'remote',
      mime: 'image/png',
      extension: 'png',
      imageUrl: 'https://cdn.example.com/preview.png',
      url: {
        finalUrl: 'https://cdn.example.com/preview.png',
        contentLength: 2_048,
        contentType: 'image/png',
        previewKind: 'image',
      },
    });

    expect(descriptor.primaryKind).toBe('image');
    expect(items.get('Size')).toBe('2.0 KB');
    expect(items.get('MIME')).toBe('image/png');
    expect(items.get('Format')).toBe('PNG');
  });
});
