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

const buildMediaInspector = (
  resolvedData?: ResolvedPreviewData
): PreviewInspectorSection | null => {
  if (!resolvedData?.media) {
    return null;
  }

  const items: PreviewInspectorSection['items'] = [];
  if (resolvedData.media.width && resolvedData.media.height) {
    items.push({
      label: 'Resolution',
      value: `${resolvedData.media.width}x${resolvedData.media.height}`,
      mono: true,
    });
  }
  if (resolvedData.media.duration) {
    items.push({ label: 'Duration', value: resolvedData.media.duration, mono: true });
  }
  if (resolvedData.media.codec) {
    items.push({ label: 'Codec', value: resolvedData.media.codec, mono: true });
  }
  if (resolvedData.media.bitrate) {
    items.push({ label: 'Bitrate', value: resolvedData.media.bitrate, mono: true });
  }
  if (resolvedData.media.sampleRate) {
    items.push({ label: 'Sample', value: resolvedData.media.sampleRate, mono: true });
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

const formatAnalysisStatus = (status: AnalysisStatus): string =>
  status === 'fallback' ? 'Fallback' : 'Matched';

const formatDiagnosticValue = (diagnostic: AnalysisDiagnostic): string =>
  `${diagnostic.severity.toUpperCase()} | ${diagnostic.code} | ${diagnostic.message}`;

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
      value: formatAnalysisStatus(status),
      mono: true,
    });
  }

  diagnostics.forEach((diagnostic, index) => {
    items.push({
      label: `Diagnostic ${index + 1}`,
      value: formatDiagnosticValue(diagnostic),
    });
  });

  return items.length > 0 ? { title: 'Analysis', items } : null;
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
    return 'url_card';
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
    if (resolvedData?.imageUrl) {
      alternateViews.push({
        key: 'resolved-image',
        label: 'Image',
        kind: 'image',
        payload: resolvedData.imageUrl,
      });
    }
    if (resolvedData?.audioUrl) {
      alternateViews.push({
        key: 'resolved-audio',
        label: 'Audio',
        kind: 'audio',
        payload: resolvedData.audioUrl,
      });
    }
    if (resolvedData?.videoUrl) {
      alternateViews.push({
        key: 'resolved-video',
        label: 'Video',
        kind: 'video',
        payload: resolvedData.videoUrl,
      });
    }
    alternateViews.push({
      key: 'url-structure',
      label: 'URL',
      kind: 'url_card',
      payload: {
        raw: semantic.rawContent ?? '',
        parts: metadata?.url_parts,
      },
    });
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
