import {
  ClipboardEntry,
  PreviewSummaryDensity,
  PreviewSummaryDescriptor,
} from '../../types/clipboard';
import { buildSemanticPreviewModel, normalizeContentPreview } from './entryPresentation';

const SUMMARY_SEPARATOR = ' | ';
const SUMMARY_FALLBACK_HEADLINES: Record<PreviewSummaryDescriptor['semanticType'], string> = {
  plain_text: 'Text',
  url: 'URL',
  ip_address: 'IP address',
  email: 'Email address',
  color: 'Color',
  code: 'Code snippet',
  command: 'Command',
  timestamp: 'Timestamp',
  json: 'JSON',
  markdown: 'Markdown',
  base64: 'Base64',
  image: 'Image',
  file: 'File',
};

const splitSummarySegments = (summary: string): string[] =>
  summary
    .split(SUMMARY_SEPARATOR)
    .map((segment) => segment.trim())
    .filter(Boolean);

const buildListSecondarySummary = (summary: string, rawContent: string | null): string => {
  const [firstSegment] = splitSummarySegments(summary);
  if (firstSegment) {
    return firstSegment;
  }

  return normalizeContentPreview(rawContent) || summary;
};

const buildRetrievalSecondarySummary = (summary: string, rawContent: string | null): string => {
  const segments = splitSummarySegments(summary);
  if (segments.length > 0) {
    return segments.join(SUMMARY_SEPARATOR);
  }

  return normalizeContentPreview(rawContent) || summary;
};

const resolveHeadline = (
  semanticType: PreviewSummaryDescriptor['semanticType'],
  headline: string,
  rawContent: string | null
): string =>
  headline.trim() ||
  normalizeContentPreview(rawContent) ||
  SUMMARY_FALLBACK_HEADLINES[semanticType];

export const buildPreviewSummary = (
  entry: ClipboardEntry,
  density: PreviewSummaryDensity
): PreviewSummaryDescriptor => {
  const model = buildSemanticPreviewModel(entry);
  const headline = resolveHeadline(model.semanticType, model.headline, model.rawContent);
  const secondarySummary =
    density === 'list'
      ? buildListSecondarySummary(model.secondarySummary, model.rawContent)
      : buildRetrievalSecondarySummary(model.secondarySummary, model.rawContent);

  return {
    density,
    semanticType: model.semanticType,
    previewIntent: model.previewIntent,
    headline,
    secondarySummary:
      secondarySummary.trim() || normalizeContentPreview(model.rawContent) || headline,
  };
};
