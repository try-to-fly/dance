import {
  ClipboardEntry,
  PreviewSummaryDensity,
  PreviewSummaryDescriptor,
} from '../../types/clipboard';
import { buildSemanticPreviewModel, normalizeContentPreview } from './entryPresentation';

const SUMMARY_SEPARATOR = ' | ';

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

export const buildPreviewSummary = (
  entry: ClipboardEntry,
  density: PreviewSummaryDensity
): PreviewSummaryDescriptor => {
  const model = buildSemanticPreviewModel(entry);
  const secondarySummary =
    density === 'list'
      ? buildListSecondarySummary(model.secondarySummary, model.rawContent)
      : buildRetrievalSecondarySummary(model.secondarySummary, model.rawContent);

  return {
    density,
    semanticType: model.semanticType,
    previewIntent: model.previewIntent,
    headline: model.headline,
    secondarySummary,
  };
};
