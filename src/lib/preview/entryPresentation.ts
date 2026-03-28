import {
  AnalysisDiagnostic,
  AnalysisStatus,
  ClipboardEntry,
  ContentMetadata,
  ContentSubType,
  EntryAnalysisMetadata,
  EntryAnalysisSnapshot,
} from '../../types/clipboard';

const DEFAULT_HEADLINE_MAX_LENGTH = 120;

const KNOWN_SUBTYPES: ContentSubType[] = [
  'plain_text',
  'url',
  'ip_address',
  'email',
  'color',
  'code',
  'command',
  'timestamp',
  'json',
  'markdown',
  'base64',
];

export interface EntrySemanticSummary {
  headline: string;
  normalizedText: string;
}

const normalizeSubType = (value?: string | null): ContentSubType => {
  if (!value) {
    return 'plain_text';
  }

  const subtype = String(value).toLowerCase();
  return KNOWN_SUBTYPES.includes(subtype as ContentSubType)
    ? (subtype as ContentSubType)
    : 'plain_text';
};

export const parseContentMetadata = (metadataString?: string | null): ContentMetadata | null => {
  if (!metadataString) {
    return null;
  }

  try {
    return JSON.parse(metadataString) as ContentMetadata;
  } catch {
    return null;
  }
};

export const getEntrySubType = (entry?: ClipboardEntry | null): ContentSubType =>
  normalizeSubType(entry?.content_subtype);

export const getEntryAnalysis = (entry?: ClipboardEntry | null): EntryAnalysisSnapshot | null =>
  entry?.analysis ?? null;

export const getEntryAnalysisSubtype = (entry?: ClipboardEntry | null): ContentSubType =>
  normalizeSubType(entry?.analysis?.subtype ?? entry?.content_subtype);

export const getEntryAnalysisMetadata = (
  entry?: ClipboardEntry | null
): EntryAnalysisMetadata | null => entry?.analysis?.metadata ?? null;

export const getEntryAnalysisStatus = (entry?: ClipboardEntry | null): AnalysisStatus | null =>
  entry?.analysis?.status ?? null;

export const getEntryAnalysisDiagnostics = (entry?: ClipboardEntry | null): AnalysisDiagnostic[] =>
  entry?.analysis?.diagnostics ?? [];

export const mapAnalysisMetadataToContentMetadata = (
  metadata?: EntryAnalysisMetadata | null
): ContentMetadata | null => {
  if (!metadata) {
    return null;
  }

  switch (metadata.kind) {
    case 'url':
      return {
        url_parts: {
          protocol: metadata.data.protocol,
          host: metadata.data.host,
          path: metadata.data.path,
          query_params: metadata.data.query_params.map((item) => [item.key, item.value]),
        },
      };
    case 'color':
      return { color_formats: metadata.data };
    case 'code':
      return {
        detected_language: metadata.data.detected_language ?? undefined,
      };
    case 'timestamp':
      return { timestamp_formats: metadata.data };
    case 'base64':
      return { base64_metadata: metadata.data };
    default:
      return null;
  }
};

export const getEntryPresentationMetadata = (
  entry?: ClipboardEntry | null
): ContentMetadata | null => {
  const analysis = getEntryAnalysis(entry);
  if (analysis) {
    return mapAnalysisMetadataToContentMetadata(analysis.metadata);
  }

  return parseContentMetadata(entry?.metadata);
};

export const getFileName = (value?: string | null): string => {
  if (!value) {
    return '';
  }

  return value.split(/[\\/]/).pop() || value;
};

export const normalizeContentPreview = (
  value?: string | null,
  maxLength = DEFAULT_HEADLINE_MAX_LENGTH
): string => {
  if (!value) {
    return '';
  }

  const normalized = value.replace(/\s+/g, ' ').trim();
  if (!normalized) {
    return '';
  }

  return normalized.length > maxLength
    ? `${normalized.slice(0, maxLength).trimEnd()}...`
    : normalized;
};

const getUrlHeadline = (content?: string | null, metadata?: ContentMetadata | null): string => {
  const host = metadata?.url_parts?.host;
  const path = metadata?.url_parts?.path;
  if (host) {
    if (path && path !== '/') {
      return `${host}${path}`;
    }
    return host;
  }

  if (!content) {
    return '';
  }

  try {
    const url = new URL(content);
    const urlPath = url.pathname && url.pathname !== '/' ? url.pathname : '';
    return `${url.host}${urlPath}` || content;
  } catch {
    return '';
  }
};

export const buildEntrySemanticSummary = (
  entry: ClipboardEntry,
  options?: {
    fallbackImageLabel?: string;
    fallbackFileLabel?: string;
    fallbackTextLabel?: string;
  }
): EntrySemanticSummary => {
  const metadata = getEntryPresentationMetadata(entry);
  const subType = getEntryAnalysisSubtype(entry);
  const contentType = entry.content_type.toLowerCase();
  const normalizedText = normalizeContentPreview(entry.content_data, 120);

  if (contentType.includes('image')) {
    return {
      headline: getFileName(entry.file_path) || options?.fallbackImageLabel || 'Image',
      normalizedText,
    };
  }

  if (contentType.includes('file')) {
    return {
      headline:
        getFileName(entry.file_path || entry.content_data) || options?.fallbackFileLabel || 'File',
      normalizedText,
    };
  }

  if (subType === 'url') {
    const urlHeadline = getUrlHeadline(entry.content_data, metadata);
    if (urlHeadline) {
      return { headline: urlHeadline, normalizedText };
    }
  }

  if (subType === 'timestamp' && metadata?.timestamp_formats?.unix_ms) {
    return {
      headline: new Date(metadata.timestamp_formats.unix_ms).toLocaleString(),
      normalizedText,
    };
  }

  if (subType === 'color' && metadata?.color_formats?.hex) {
    return { headline: metadata.color_formats.hex, normalizedText };
  }

  if (subType === 'base64') {
    const hint = metadata?.base64_metadata?.content_hint;
    if (hint) {
      return { headline: `Base64 (${hint})`, normalizedText };
    }

    return { headline: 'Base64', normalizedText };
  }

  return {
    headline: normalizedText || options?.fallbackTextLabel || 'Text',
    normalizedText,
  };
};
