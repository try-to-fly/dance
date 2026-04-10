import {
  AnalysisDiagnostic,
  AnalysisStatus,
  ClipboardEntry,
  ContentMetadata,
  ContentSubType,
  PreviewDescriptor,
  PreviewInspectorSection,
  PreviewKind,
  ResolvedPreviewData,
} from '../../types/clipboard';
import {
  buildSemanticPreviewModel,
  getEntryAnalysisDiagnostics,
  getEntryAnalysisStatus,
  getEntryAnalysisSubtype,
  getEntryPresentationMetadata,
} from './entryPresentation';
import { formatAnalysisReasonCopy, formatAnalysisStatusCopy } from './analysisPresentation';

export interface PreviewLabelSet {
  unknown: string;
  image: string;
  file: string;
  text: string;
  base64: string;
  subtypeLabels?: Partial<Record<ContentSubType, string>>;
}

const hasResolvedJsonContent = (
  resolvedData?: Pick<ResolvedPreviewData, 'jsonContent'>
): resolvedData is Pick<ResolvedPreviewData, 'jsonContent'> & { jsonContent: unknown } =>
  resolvedData?.jsonContent !== undefined;

const hasCopyableDecodedContent = (resolvedData?: ResolvedPreviewData) =>
  Boolean(
    resolvedData?.base64?.textPreview ||
      resolvedData?.textContent ||
      hasResolvedJsonContent(resolvedData)
  );

const stringifyUnknown = (value: unknown) => {
  if (typeof value === 'string') {
    return value;
  }

  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
};

const shouldIncludeRawAlternateView = (
  primaryKind: PreviewKind,
  subType: ContentSubType
): boolean => {
  if (subType === 'base64') {
    return true;
  }

  if (subType === 'url') {
    return false;
  }

  return !['plain_text', 'code', 'markdown'].includes(primaryKind);
};

const shouldIncludeResolvedJsonAlternateView = (
  primaryKind: PreviewKind,
  subType: ContentSubType
): boolean => primaryKind !== 'json' && subType !== 'url';

const shouldIncludeResolvedTextAlternateView = (
  primaryKind: PreviewKind,
  subType: ContentSubType
): boolean => !['plain_text', 'code', 'markdown'].includes(primaryKind) && subType !== 'url';

const buildUrlInspector = (metadata: ContentMetadata | null): PreviewInspectorSection | null => {
  if (!metadata?.url_parts) {
    return null;
  }

  const { protocol, host, path, query_params } = metadata.url_parts;
  const items = [
    { label: 'Protocol', value: protocol, mono: true },
    { label: 'Host', value: host, mono: true },
  ];

  if (path) {
    items.push({ label: 'Path', value: path, mono: true });
  }

  if (query_params.length > 0) {
    items.push({ label: 'Query', value: `${query_params.length}`, mono: true });
  }

  return { title: 'URL', items };
};

const MEDIA_PRIMARY_KINDS = new Set<PreviewKind>(['image', 'audio', 'video']);

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

const formatMediaFormat = (value?: string | null) => {
  if (!value) {
    return '';
  }

  return value.replace(/^\./, '').toUpperCase();
};

const buildMediaInspector = (
  resolvedData?: ResolvedPreviewData
): PreviewInspectorSection | null => {
  const media = resolvedData?.media;
  const isUrlMediaPreview = MEDIA_PRIMARY_KINDS.has(
    resolvedData?.url?.previewKind ?? 'unsupported'
  );

  if (!media && !isUrlMediaPreview) {
    return null;
  }

  const items: PreviewInspectorSection['items'] = [];
  if (media?.width && media.height) {
    items.push({
      label: 'Resolution',
      value: `${media.width}x${media.height}`,
      mono: true,
    });
  }

  const resolvedSizeBytes =
    media?.sizeBytes ?? resolvedData?.sizeBytes ?? resolvedData?.url?.contentLength;
  const formattedSize = formatBinarySize(resolvedSizeBytes);
  if (formattedSize) {
    items.push({ label: 'Size', value: formattedSize, mono: true });
  }

  const resolvedMime = resolvedData?.mime || resolvedData?.url?.contentType;
  if (resolvedMime) {
    items.push({ label: 'MIME', value: resolvedMime, mono: true });
  }

  const resolvedFormat = formatMediaFormat(media?.format ?? resolvedData?.extension);
  if (resolvedFormat) {
    items.push({ label: 'Format', value: resolvedFormat, mono: true });
  }

  if (media?.duration) {
    items.push({ label: 'Duration', value: media.duration, mono: true });
  }
  if (media?.fps) {
    items.push({ label: 'FPS', value: media.fps, mono: true });
  }
  if (media?.codec) {
    items.push({ label: 'Codec', value: media.codec, mono: true });
  }
  if (media?.bitrate) {
    items.push({ label: 'Bitrate', value: media.bitrate, mono: true });
  }
  if (media?.sampleRate) {
    items.push({ label: 'Sample Rate', value: media.sampleRate, mono: true });
  }

  if (items.length === 0) {
    return null;
  }

  return { title: 'Media', items };
};

const buildBase64Inspector = (
  metadata: ContentMetadata | null,
  resolvedData?: ResolvedPreviewData
): PreviewInspectorSection | null => {
  const base64Meta = metadata?.base64_metadata;
  const resolvedBase64 = resolvedData?.base64;
  if (!base64Meta && !resolvedBase64) {
    return null;
  }

  const items: PreviewInspectorSection['items'] = [];
  if (base64Meta?.encoded_size) {
    items.push({ label: 'Encoded', value: `${base64Meta.encoded_size}`, mono: true });
  }
  if (base64Meta?.estimated_original_size) {
    items.push({
      label: 'Decoded',
      value: `${base64Meta.estimated_original_size}`,
      mono: true,
    });
  }
  if (resolvedBase64?.decodedKind) {
    items.push({ label: 'Kind', value: resolvedBase64.decodedKind, mono: true });
  }
  if (resolvedBase64?.mime) {
    items.push({ label: 'MIME', value: resolvedBase64.mime, mono: true });
  }

  if (items.length === 0) {
    return null;
  }

  return { title: 'Base64', items };
};

const URL_CONTENT_PRIMARY_KINDS = new Set<PreviewKind>([
  'image',
  'audio',
  'video',
  'json',
  'markdown',
  'code',
  'plain_text',
]);

const resolveUrlPrimaryKind = (resolvedData?: ResolvedPreviewData): PreviewKind | null => {
  const resolvedPreviewKind = resolvedData?.url?.previewKind;

  if (resolvedPreviewKind && URL_CONTENT_PRIMARY_KINDS.has(resolvedPreviewKind)) {
    if (resolvedPreviewKind === 'image' && resolvedData?.imageUrl) {
      return 'image';
    }
    if (resolvedPreviewKind === 'audio' && resolvedData?.audioUrl) {
      return 'audio';
    }
    if (resolvedPreviewKind === 'video' && resolvedData?.videoUrl) {
      return 'video';
    }
    if (
      resolvedPreviewKind === 'json' &&
      (hasResolvedJsonContent(resolvedData) || resolvedData?.textContent)
    ) {
      return 'json';
    }
    if (
      ['markdown', 'code', 'plain_text'].includes(resolvedPreviewKind) &&
      resolvedData?.textContent
    ) {
      return resolvedPreviewKind;
    }
  }

  if (resolvedData?.imageUrl) {
    return 'image';
  }
  if (resolvedData?.audioUrl) {
    return 'audio';
  }
  if (resolvedData?.videoUrl) {
    return 'video';
  }
  if (hasResolvedJsonContent(resolvedData)) {
    return 'json';
  }
  if (resolvedData?.textContent) {
    return 'plain_text';
  }

  return null;
};

const buildAnalysisInspector = (
  status: AnalysisStatus | null,
  diagnostics: AnalysisDiagnostic[]
): PreviewInspectorSection | null => {
  if (status !== 'fallback' && diagnostics.length === 0) {
    return null;
  }

  const items: PreviewInspectorSection['items'] = [];

  if (status) {
    items.push({
      label: 'Status',
      value: formatAnalysisStatusCopy(status),
    });
  }

  diagnostics.forEach((diagnostic, index) => {
    items.push({
      label: diagnostics.length > 1 ? `Reason ${index + 1}` : 'Reason',
      value: formatAnalysisReasonCopy(diagnostic),
    });
  });

  return items.length > 0 ? { title: 'Detection', items } : null;
};

const resolvePrimaryKind = (
  semanticType: ReturnType<typeof buildSemanticPreviewModel>['semanticType'],
  resolvedData?: ResolvedPreviewData
): PreviewKind => {
  if (semanticType === 'image') {
    return 'image';
  }
  if (semanticType === 'file') {
    return 'file_card';
  }

  if (semanticType === 'url') {
    return resolveUrlPrimaryKind(resolvedData) ?? 'url_card';
  }

  if (semanticType === 'base64') {
    const hasInlineMedia =
      Boolean(resolvedData?.base64?.dataUrl) ||
      Boolean(resolvedData?.imageUrl) ||
      Boolean(resolvedData?.audioUrl) ||
      Boolean(resolvedData?.videoUrl);

    switch (resolvedData?.base64?.decodedKind) {
      case 'json':
        return 'json';
      case 'image':
        return hasInlineMedia ? 'image' : 'base64_binary';
      case 'audio':
        return hasInlineMedia ? 'audio' : 'base64_binary';
      case 'video':
        return hasInlineMedia ? 'video' : 'base64_binary';
      case 'text':
        return 'base64_text';
      case 'binary':
        return 'base64_binary';
      default:
        return 'base64_text';
    }
  }

  if (semanticType === 'json') {
    return 'json';
  }
  if (semanticType === 'code') {
    return 'code';
  }
  if (semanticType === 'markdown') {
    return 'markdown';
  }
  if (semanticType === 'ip_address') {
    return 'ip_card';
  }
  if (semanticType === 'email') {
    return 'email_card';
  }
  if (semanticType === 'color') {
    return 'color_card';
  }
  if (semanticType === 'timestamp') {
    return 'timestamp_card';
  }
  if (semanticType === 'command') {
    return 'code';
  }

  return 'plain_text';
};

const resolveTypeLabel = (
  semanticType: ReturnType<typeof buildSemanticPreviewModel>['semanticType'],
  labels: PreviewLabelSet
): string => {
  if (semanticType === 'image') {
    return labels.image;
  }
  if (semanticType === 'file') {
    return labels.file;
  }
  if (semanticType === 'base64') {
    return labels.base64;
  }
  const subtypeLabel = labels.subtypeLabels?.[semanticType as ContentSubType];
  if (subtypeLabel) {
    return subtypeLabel;
  }
  return labels.text;
};

export const buildPreviewDescriptor = ({
  entry,
  resolvedData,
  labels,
}: {
  entry: ClipboardEntry;
  resolvedData?: ResolvedPreviewData;
  labels: PreviewLabelSet;
}): PreviewDescriptor => {
  const metadata = getEntryPresentationMetadata(entry);
  const subType = getEntryAnalysisSubtype(entry);
  const analysisStatus = getEntryAnalysisStatus(entry);
  const diagnostics = getEntryAnalysisDiagnostics(entry);
  const semantic = buildSemanticPreviewModel(entry, {
    fallbackImageLabel: labels.image,
    fallbackFileLabel: labels.file,
    fallbackTextLabel: labels.text,
  });
  const primaryKind = resolvePrimaryKind(semantic.semanticType, resolvedData);
  const typeLabel = resolveTypeLabel(semantic.semanticType, labels);

  const alternateViews: PreviewDescriptor['alternateViews'] = [];
  if (
    semantic.supportsRawView &&
    semantic.rawContent &&
    shouldIncludeRawAlternateView(primaryKind, subType)
  ) {
    alternateViews.push({
      key: 'raw',
      label: 'Raw',
      kind: 'raw',
      payload: semantic.rawContent,
    });
  }
  if (
    resolvedData?.textContent &&
    resolvedData.textContent !== semantic.rawContent &&
    shouldIncludeResolvedTextAlternateView(primaryKind, subType)
  ) {
    alternateViews.push({
      key: 'resolved-text',
      label: 'Resolved',
      kind: 'plain_text',
      payload: resolvedData.textContent,
    });
  }
  if (
    resolvedData?.jsonContent !== undefined &&
    shouldIncludeResolvedJsonAlternateView(primaryKind, subType)
  ) {
    alternateViews.push({
      key: 'resolved-json',
      label: 'JSON',
      kind: 'json',
      payload: stringifyUnknown(resolvedData.jsonContent),
    });
  }
  if (subType === 'base64' && resolvedData?.base64?.textPreview) {
    alternateViews.push({
      key: 'decoded',
      label: 'Decoded',
      kind: 'plain_text',
      payload: resolvedData.base64.textPreview,
    });
  }
  if (subType === 'url') {
    if (resolvedData?.imageUrl && primaryKind !== 'image') {
      alternateViews.push({
        key: 'resolved-image',
        label: 'Image',
        kind: 'image',
        payload: resolvedData.imageUrl,
      });
    }
    if (resolvedData?.audioUrl && primaryKind !== 'audio') {
      alternateViews.push({
        key: 'resolved-audio',
        label: 'Audio',
        kind: 'audio',
        payload: resolvedData.audioUrl,
      });
    }
    if (resolvedData?.videoUrl && primaryKind !== 'video') {
      alternateViews.push({
        key: 'resolved-video',
        label: 'Video',
        kind: 'video',
        payload: resolvedData.videoUrl,
      });
    }
  }

  const inspectorSections: PreviewInspectorSection[] = [];
  const urlInspector = buildUrlInspector(metadata);
  const mediaInspector = buildMediaInspector(resolvedData);
  const base64Inspector = buildBase64Inspector(metadata, resolvedData);
  const analysisInspector = buildAnalysisInspector(analysisStatus, diagnostics);
  if (urlInspector) {
    inspectorSections.push(urlInspector);
  }
  if (mediaInspector) {
    inspectorSections.push(mediaInspector);
  }
  if (base64Inspector) {
    inspectorSections.push(base64Inspector);
  }
  if (analysisInspector) {
    inspectorSections.push(analysisInspector);
  }

  const primaryPayload = {
    entry,
    subType,
    metadata,
    resolvedData,
  };

  const actions: PreviewDescriptor['actions'] = [];
  if (semantic.supportsRawView && semantic.rawContent) {
    actions.push('copy_raw');
  }
  if (subType === 'base64' && hasCopyableDecodedContent(resolvedData)) {
    actions.push('copy_decoded');
  }
  actions.push('paste');
  if (subType === 'url') {
    actions.push('open_url');
  }
  if (entry.file_path) {
    actions.push('open_file');
  }

  const badges: PreviewDescriptor['badges'] = [];
  if (analysisStatus === 'fallback') {
    badges.push({
      label: 'Fallback',
      tone: 'warning',
    });
  }

  return {
    headline: semantic.headline,
    typeLabel,
    badges,
    primaryKind,
    primaryPayload,
    inspectorSections,
    alternateViews,
    actions,
  };
};
