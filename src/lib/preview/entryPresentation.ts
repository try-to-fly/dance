import {
  AnalysisUrlMetadata,
  AnalysisDiagnostic,
  AnalysisStatus,
  Base64Metadata,
  ClipboardEntry,
  CodeAnalysisMetadata,
  ColorFormats,
  CommandAnalysisMetadata,
  ContentMetadata,
  ContentSubType,
  EmailAnalysisMetadata,
  EntryAnalysisMetadata,
  EntryAnalysisSnapshot,
  IpAddressAnalysisMetadata,
  JsonAnalysisMetadata,
  MarkdownAnalysisMetadata,
  PlainTextAnalysisMetadata,
  PreviewIntent,
  SemanticPreviewModel,
  TimestampFormats,
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

interface SemanticPreviewOptions {
  fallbackImageLabel?: string;
  fallbackFileLabel?: string;
  fallbackTextLabel?: string;
}

export interface EntrySemanticSummary {
  headline: string;
  normalizedText: string;
}

type AnalysisKind = EntryAnalysisMetadata['kind'];

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

function getAnalysisMetadataOfKind(
  metadata: EntryAnalysisMetadata | null,
  kind: 'plain_text'
): PlainTextAnalysisMetadata | null;
function getAnalysisMetadataOfKind(
  metadata: EntryAnalysisMetadata | null,
  kind: 'url'
): AnalysisUrlMetadata | null;
function getAnalysisMetadataOfKind(
  metadata: EntryAnalysisMetadata | null,
  kind: 'ip_address'
): IpAddressAnalysisMetadata | null;
function getAnalysisMetadataOfKind(
  metadata: EntryAnalysisMetadata | null,
  kind: 'email'
): EmailAnalysisMetadata | null;
function getAnalysisMetadataOfKind(
  metadata: EntryAnalysisMetadata | null,
  kind: 'color'
): ColorFormats | null;
function getAnalysisMetadataOfKind(
  metadata: EntryAnalysisMetadata | null,
  kind: 'code'
): CodeAnalysisMetadata | null;
function getAnalysisMetadataOfKind(
  metadata: EntryAnalysisMetadata | null,
  kind: 'command'
): CommandAnalysisMetadata | null;
function getAnalysisMetadataOfKind(
  metadata: EntryAnalysisMetadata | null,
  kind: 'timestamp'
): TimestampFormats | null;
function getAnalysisMetadataOfKind(
  metadata: EntryAnalysisMetadata | null,
  kind: 'json'
): JsonAnalysisMetadata | null;
function getAnalysisMetadataOfKind(
  metadata: EntryAnalysisMetadata | null,
  kind: 'markdown'
): MarkdownAnalysisMetadata | null;
function getAnalysisMetadataOfKind(
  metadata: EntryAnalysisMetadata | null,
  kind: 'base64'
): Base64Metadata | null;
function getAnalysisMetadataOfKind(metadata: EntryAnalysisMetadata | null, kind: AnalysisKind) {
  if (!metadata || metadata.kind !== kind) {
    return null;
  }

  return metadata.data;
}

const joinSummaryParts = (parts: Array<string | null | undefined>): string =>
  parts.filter((value): value is string => Boolean(value)).join(' | ');

const formatCount = (count: number, singular: string, plural = `${singular}s`): string =>
  `${count} ${count === 1 ? singular : plural}`;

const countLines = (value?: string | null): number => {
  if (!value) {
    return 0;
  }

  return value.split(/\r?\n/).length;
};

const getFirstMeaningfulLine = (value?: string | null): string => {
  if (!value) {
    return '';
  }

  const line = value
    .split(/\r?\n/)
    .map((item) => item.trim())
    .find(Boolean);

  return line ?? '';
};

const stripMarkdownLead = (value: string): string =>
  value.replace(/^\s*(?:#{1,6}|[-*+]|\d+\.)\s+/, '').trim();

const formatUpperToken = (value?: string | null): string => {
  if (!value) {
    return '';
  }

  return value.replace(/^\./, '').toUpperCase();
};

const buildLineAndCharacterSummary = (lineCount: number, charCount: number): string =>
  joinSummaryParts([formatCount(lineCount, 'line'), formatCount(charCount, 'char')]);

const buildSemanticModel = ({
  semanticType,
  previewIntent,
  headline,
  secondarySummary,
  rawContent,
  usesWorkbench = false,
  fallbackHeadline,
  fallbackSecondary,
}: {
  semanticType: SemanticPreviewModel['semanticType'];
  previewIntent: PreviewIntent;
  headline?: string | null;
  secondarySummary?: string | null;
  rawContent: string | null;
  usesWorkbench?: boolean;
  fallbackHeadline: string;
  fallbackSecondary: string;
}): SemanticPreviewModel => {
  const resolvedHeadline = headline?.trim() || fallbackHeadline;
  const resolvedSecondary = secondarySummary?.trim() || fallbackSecondary || resolvedHeadline;

  return {
    semanticType,
    previewIntent,
    headline: resolvedHeadline,
    secondarySummary: resolvedSecondary,
    rawContent,
    supportsRawView: Boolean(rawContent),
    usesWorkbench,
  };
};

const getUrlPartsFromContent = (content?: string | null) => {
  if (!content) {
    return null;
  }

  try {
    const url = new URL(content);
    return {
      protocol: url.protocol.replace(/:$/, ''),
      host: url.host,
      path: url.pathname || '/',
      queryCount: Array.from(url.searchParams.entries()).length,
    };
  } catch {
    return null;
  }
};

const getUrlPreviewInfo = (
  metadata?: ContentMetadata | null,
  content?: string | null
): {
  protocol: string;
  host: string;
  path: string;
  queryCount: number;
} | null => {
  if (metadata?.url_parts) {
    return {
      protocol: metadata.url_parts.protocol,
      host: metadata.url_parts.host,
      path: metadata.url_parts.path || '/',
      queryCount: metadata.url_parts.query_params.length,
    };
  }

  return getUrlPartsFromContent(content);
};

const getUrlHeadline = (content?: string | null, metadata?: ContentMetadata | null): string => {
  const info = getUrlPreviewInfo(metadata, content);
  if (!info?.host) {
    return '';
  }

  if (info.path && info.path !== '/') {
    return `${info.host}${info.path}`;
  }

  return info.host;
};

const buildPlainTextPreviewModel = ({
  entry,
  analysisMetadata,
  options,
}: {
  entry: ClipboardEntry;
  analysisMetadata: EntryAnalysisMetadata | null;
  options?: SemanticPreviewOptions;
}): SemanticPreviewModel => {
  const rawContent = entry.content_data ?? null;
  const textMetadata = getAnalysisMetadataOfKind(analysisMetadata, 'plain_text');
  const lineCount = textMetadata?.line_count ?? countLines(rawContent);
  const charCount = textMetadata?.char_count ?? rawContent?.length ?? 0;
  const summary = buildLineAndCharacterSummary(lineCount, charCount);
  const headline = normalizeContentPreview(rawContent);

  return buildSemanticModel({
    semanticType: 'plain_text',
    previewIntent: 'plain_text_summary',
    headline,
    secondarySummary: summary,
    rawContent,
    fallbackHeadline: options?.fallbackTextLabel || 'Text',
    fallbackSecondary: summary || options?.fallbackTextLabel || 'Text',
  });
};

const buildUrlPreviewModel = ({
  entry,
  metadata,
}: {
  entry: ClipboardEntry;
  metadata: ContentMetadata | null;
}): SemanticPreviewModel => {
  const rawContent = entry.content_data ?? null;
  const info = getUrlPreviewInfo(metadata, rawContent);
  const headline = getUrlHeadline(rawContent, metadata);
  const querySummary =
    info && info.queryCount > 0 ? formatCount(info.queryCount, 'query param') : null;
  const secondarySummary = joinSummaryParts([info?.protocol?.toUpperCase(), querySummary]);

  return buildSemanticModel({
    semanticType: 'url',
    previewIntent: 'url_structured',
    headline,
    secondarySummary,
    rawContent,
    fallbackHeadline: normalizeContentPreview(rawContent) || 'URL',
    fallbackSecondary: normalizeContentPreview(rawContent) || 'URL',
  });
};

const buildIpPreviewModel = ({
  entry,
  analysisMetadata,
}: {
  entry: ClipboardEntry;
  analysisMetadata: EntryAnalysisMetadata | null;
}): SemanticPreviewModel => {
  const rawContent = entry.content_data ?? null;
  const ipMetadata = getAnalysisMetadataOfKind(analysisMetadata, 'ip_address');
  const version = ipMetadata?.version === 'v6' ? 'IPv6' : 'IPv4';
  const scope = ipMetadata?.is_loopback
    ? 'Loopback'
    : ipMetadata
      ? ipMetadata.is_private
        ? 'Private'
        : 'Public'
      : null;

  return buildSemanticModel({
    semanticType: 'ip_address',
    previewIntent: 'ip_structured',
    headline: normalizeContentPreview(rawContent),
    secondarySummary: joinSummaryParts([version, scope]),
    rawContent,
    fallbackHeadline: 'IP address',
    fallbackSecondary: version,
  });
};

const buildEmailPreviewModel = ({
  entry,
  analysisMetadata,
}: {
  entry: ClipboardEntry;
  analysisMetadata: EntryAnalysisMetadata | null;
}): SemanticPreviewModel => {
  const rawContent = entry.content_data ?? null;
  const emailMetadata = getAnalysisMetadataOfKind(analysisMetadata, 'email');
  const fallbackDomain = rawContent?.split('@')[1]?.trim();
  const domain = emailMetadata?.domain || fallbackDomain;

  return buildSemanticModel({
    semanticType: 'email',
    previewIntent: 'email_structured',
    headline: normalizeContentPreview(rawContent),
    secondarySummary: domain ? `Domain ${domain}` : null,
    rawContent,
    fallbackHeadline: 'Email address',
    fallbackSecondary: normalizeContentPreview(rawContent) || 'Email address',
  });
};

const buildColorPreviewModel = ({
  entry,
  metadata,
  analysisMetadata,
}: {
  entry: ClipboardEntry;
  metadata: ContentMetadata | null;
  analysisMetadata: EntryAnalysisMetadata | null;
}): SemanticPreviewModel => {
  const rawContent = entry.content_data ?? null;
  const analysisColor = getAnalysisMetadataOfKind(analysisMetadata, 'color');
  const colorFormats = analysisColor ?? metadata?.color_formats;
  const secondarySummary = joinSummaryParts([
    colorFormats?.rgb || colorFormats?.rgba,
    colorFormats?.hsl,
  ]);

  return buildSemanticModel({
    semanticType: 'color',
    previewIntent: 'color_structured',
    headline: colorFormats?.hex || normalizeContentPreview(rawContent),
    secondarySummary,
    rawContent,
    fallbackHeadline: 'Color',
    fallbackSecondary: normalizeContentPreview(rawContent) || 'Color',
  });
};

const buildCodePreviewModel = ({
  entry,
  analysisMetadata,
}: {
  entry: ClipboardEntry;
  analysisMetadata: EntryAnalysisMetadata | null;
}): SemanticPreviewModel => {
  const rawContent = entry.content_data ?? null;
  const codeMetadata = getAnalysisMetadataOfKind(analysisMetadata, 'code');
  const headline = normalizeContentPreview(getFirstMeaningfulLine(rawContent));
  const secondarySummary = joinSummaryParts([
    codeMetadata?.detected_language || null,
    formatCount(codeMetadata?.line_count ?? countLines(rawContent), 'line'),
  ]);

  return buildSemanticModel({
    semanticType: 'code',
    previewIntent: 'code_workbench',
    headline,
    secondarySummary,
    rawContent,
    usesWorkbench: true,
    fallbackHeadline: 'Code snippet',
    fallbackSecondary: secondarySummary || 'Code snippet',
  });
};

const buildCommandPreviewModel = ({
  entry,
  analysisMetadata,
}: {
  entry: ClipboardEntry;
  analysisMetadata: EntryAnalysisMetadata | null;
}): SemanticPreviewModel => {
  const rawContent = entry.content_data ?? null;
  const commandMetadata = getAnalysisMetadataOfKind(analysisMetadata, 'command');
  const secondarySummary = joinSummaryParts([
    commandMetadata?.shell_family || null,
    commandMetadata?.command_name ? `command ${commandMetadata.command_name}` : null,
    commandMetadata?.has_pipeline ? 'Pipeline' : null,
    commandMetadata?.has_sudo_prefix ? 'sudo' : null,
  ]);

  return buildSemanticModel({
    semanticType: 'command',
    previewIntent: 'command_workbench',
    headline: normalizeContentPreview(getFirstMeaningfulLine(rawContent)),
    secondarySummary,
    rawContent,
    usesWorkbench: true,
    fallbackHeadline: 'Command',
    fallbackSecondary: secondarySummary || 'Command',
  });
};

const buildTimestampPreviewModel = ({
  entry,
  metadata,
  analysisMetadata,
}: {
  entry: ClipboardEntry;
  metadata: ContentMetadata | null;
  analysisMetadata: EntryAnalysisMetadata | null;
}): SemanticPreviewModel => {
  const rawContent = entry.content_data ?? null;
  const timestampMetadata =
    getAnalysisMetadataOfKind(analysisMetadata, 'timestamp') ?? metadata?.timestamp_formats ?? null;
  const headline = timestampMetadata?.unix_ms
    ? new Date(timestampMetadata.unix_ms).toLocaleString()
    : timestampMetadata?.date_string ||
      timestampMetadata?.iso8601 ||
      normalizeContentPreview(rawContent);
  const secondarySummary =
    timestampMetadata?.iso8601 ||
    timestampMetadata?.date_string ||
    normalizeContentPreview(rawContent);

  return buildSemanticModel({
    semanticType: 'timestamp',
    previewIntent: 'timestamp_structured',
    headline,
    secondarySummary,
    rawContent,
    fallbackHeadline: 'Timestamp',
    fallbackSecondary: normalizeContentPreview(rawContent) || 'Timestamp',
  });
};

const buildJsonPreviewModel = ({
  entry,
  analysisMetadata,
}: {
  entry: ClipboardEntry;
  analysisMetadata: EntryAnalysisMetadata | null;
}): SemanticPreviewModel => {
  const rawContent = entry.content_data ?? null;
  const jsonMetadata = getAnalysisMetadataOfKind(analysisMetadata, 'json');
  const keySummary =
    jsonMetadata?.key_count !== undefined && jsonMetadata?.key_count !== null
      ? formatCount(jsonMetadata.key_count, 'key')
      : null;
  const secondarySummary = joinSummaryParts([
    keySummary,
    rawContent ? formatCount(rawContent.length, 'char') : null,
  ]);

  return buildSemanticModel({
    semanticType: 'json',
    previewIntent: 'json_structured',
    headline: jsonMetadata ? `JSON ${jsonMetadata.root_kind}` : 'JSON',
    secondarySummary,
    rawContent,
    fallbackHeadline: 'JSON',
    fallbackSecondary: normalizeContentPreview(rawContent) || 'JSON',
  });
};

const buildMarkdownPreviewModel = ({
  entry,
  analysisMetadata,
}: {
  entry: ClipboardEntry;
  analysisMetadata: EntryAnalysisMetadata | null;
}): SemanticPreviewModel => {
  const rawContent = entry.content_data ?? null;
  const markdownMetadata = getAnalysisMetadataOfKind(analysisMetadata, 'markdown');
  const headingMatch = rawContent?.match(/^\s*#{1,6}\s+(.+)$/m);
  const headline = headingMatch
    ? normalizeContentPreview(headingMatch[1])
    : normalizeContentPreview(stripMarkdownLead(getFirstMeaningfulLine(rawContent)));
  const secondarySummary = joinSummaryParts([
    markdownMetadata?.has_heading ? 'Heading' : null,
    markdownMetadata?.has_list ? 'List' : null,
    markdownMetadata?.has_code_fence ? 'Code fence' : null,
    markdownMetadata?.has_link ? 'Link' : null,
  ]);

  return buildSemanticModel({
    semanticType: 'markdown',
    previewIntent: 'markdown_structured',
    headline,
    secondarySummary,
    rawContent,
    fallbackHeadline: 'Markdown',
    fallbackSecondary: normalizeContentPreview(rawContent) || 'Markdown',
  });
};

const buildBase64PreviewModel = ({
  entry,
  metadata,
  analysisMetadata,
}: {
  entry: ClipboardEntry;
  metadata: ContentMetadata | null;
  analysisMetadata: EntryAnalysisMetadata | null;
}): SemanticPreviewModel => {
  const rawContent = entry.content_data ?? null;
  const base64Metadata =
    getAnalysisMetadataOfKind(analysisMetadata, 'base64') ?? metadata?.base64_metadata ?? null;
  const encodedSize = base64Metadata?.encoded_size ?? rawContent?.length ?? null;
  const secondarySummary = joinSummaryParts([
    encodedSize ? formatCount(encodedSize, 'char') : null,
    base64Metadata?.estimated_original_size
      ? `${base64Metadata.estimated_original_size} bytes decoded`
      : null,
  ]);
  const headline = base64Metadata?.content_hint
    ? `Base64 (${base64Metadata.content_hint})`
    : 'Base64';

  return buildSemanticModel({
    semanticType: 'base64',
    previewIntent: 'base64_summary',
    headline,
    secondarySummary,
    rawContent,
    fallbackHeadline: 'Base64',
    fallbackSecondary: normalizeContentPreview(rawContent) || 'Base64',
  });
};

const buildImagePreviewModel = ({
  entry,
  legacyMetadata,
  options,
}: {
  entry: ClipboardEntry;
  legacyMetadata: ContentMetadata | null;
  options?: SemanticPreviewOptions;
}): SemanticPreviewModel => {
  const rawContent = entry.content_data ?? null;
  const imageMetadata = legacyMetadata?.image_metadata;
  const headline =
    getFileName(entry.file_path) ||
    getFileName(rawContent) ||
    options?.fallbackImageLabel ||
    'Image';
  const secondarySummary = joinSummaryParts([
    formatUpperToken(imageMetadata?.format),
    imageMetadata?.width && imageMetadata?.height
      ? `${imageMetadata.width}x${imageMetadata.height}`
      : null,
  ]);

  return buildSemanticModel({
    semanticType: 'image',
    previewIntent: 'image_asset',
    headline,
    secondarySummary,
    rawContent,
    fallbackHeadline: options?.fallbackImageLabel || 'Image',
    fallbackSecondary: normalizeContentPreview(rawContent) || headline,
  });
};

const buildFilePreviewModel = ({
  entry,
  legacyMetadata,
  options,
}: {
  entry: ClipboardEntry;
  legacyMetadata: ContentMetadata | null;
  options?: SemanticPreviewOptions;
}): SemanticPreviewModel => {
  const rawContent = entry.content_data ?? null;
  const fileMetadata = legacyMetadata?.file_metadata;
  const headline =
    getFileName(entry.file_path) ||
    getFileName(rawContent) ||
    fileMetadata?.name ||
    options?.fallbackFileLabel ||
    'File';
  const secondarySummary = joinSummaryParts([
    formatUpperToken(fileMetadata?.extension),
    fileMetadata?.mime,
  ]);

  return buildSemanticModel({
    semanticType: 'file',
    previewIntent: 'file_asset',
    headline,
    secondarySummary,
    rawContent,
    fallbackHeadline: options?.fallbackFileLabel || 'File',
    fallbackSecondary: normalizeContentPreview(rawContent) || headline,
  });
};

export const buildSemanticPreviewModel = (
  entry: ClipboardEntry,
  options?: SemanticPreviewOptions
): SemanticPreviewModel => {
  const contentType = entry.content_type.toLowerCase();
  const analysisMetadata = getEntryAnalysisMetadata(entry);
  const presentationMetadata = getEntryPresentationMetadata(entry);
  const legacyMetadata = parseContentMetadata(entry.metadata);
  const subType = getEntryAnalysisSubtype(entry);

  if (contentType.includes('image')) {
    return buildImagePreviewModel({ entry, legacyMetadata, options });
  }

  if (contentType.includes('file')) {
    return buildFilePreviewModel({ entry, legacyMetadata, options });
  }

  switch (subType) {
    case 'plain_text':
      return buildPlainTextPreviewModel({ entry, analysisMetadata, options });
    case 'url':
      return buildUrlPreviewModel({ entry, metadata: presentationMetadata });
    case 'ip_address':
      return buildIpPreviewModel({ entry, analysisMetadata });
    case 'email':
      return buildEmailPreviewModel({ entry, analysisMetadata });
    case 'color':
      return buildColorPreviewModel({
        entry,
        metadata: presentationMetadata,
        analysisMetadata,
      });
    case 'code':
      return buildCodePreviewModel({ entry, analysisMetadata });
    case 'command':
      return buildCommandPreviewModel({ entry, analysisMetadata });
    case 'timestamp':
      return buildTimestampPreviewModel({
        entry,
        metadata: presentationMetadata,
        analysisMetadata,
      });
    case 'json':
      return buildJsonPreviewModel({ entry, analysisMetadata });
    case 'markdown':
      return buildMarkdownPreviewModel({ entry, analysisMetadata });
    case 'base64':
      return buildBase64PreviewModel({
        entry,
        metadata: presentationMetadata,
        analysisMetadata,
      });
  }
};

export const buildEntrySemanticSummary = (
  entry: ClipboardEntry,
  options?: SemanticPreviewOptions
): EntrySemanticSummary => {
  const semanticModel = buildSemanticPreviewModel(entry, options);

  return {
    headline: semanticModel.headline,
    normalizedText: normalizeContentPreview(entry.content_data, 120),
  };
};
