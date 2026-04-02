import { AnalysisDiagnostic, AnalysisDiagnosticCode, AnalysisStatus } from '../../types/clipboard';

const ANALYSIS_REASON_COPY: Record<AnalysisDiagnosticCode, string> = {
  heuristic_fallback: 'No structured preview was detected',
  json_malformed: 'Looks like JSON, but the format is incomplete',
  base64_malformed: 'Looks like Base64, but decoding failed',
  url_malformed: 'Looks like a URL, but the address is incomplete',
  metadata_unavailable: 'Structured details were not available',
};

export const formatAnalysisStatusCopy = (status: AnalysisStatus | null): string => {
  if (status === 'fallback') {
    return 'Shown as plain text';
  }

  if (status === 'matched') {
    return 'Structured preview matched';
  }

  return '';
};

export const formatAnalysisReasonCopy = (
  diagnostic?: AnalysisDiagnostic | null,
  fallback = 'Shown as plain text'
): string => {
  if (!diagnostic) {
    return fallback;
  }

  return ANALYSIS_REASON_COPY[diagnostic.code] ?? fallback;
};
