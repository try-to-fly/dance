import { describe, expect, it } from 'vitest';
import {
  buildEntrySemanticSummary,
  getEntryAnalysisDiagnostics,
  getEntryAnalysisStatus,
  getEntryAnalysisSubtype,
  getEntryPresentationMetadata,
} from './entryPresentation';
import { ClipboardEntry, EntryAnalysisSnapshot } from '../../types/clipboard';

const baseEntry: ClipboardEntry = {
  id: 'entry-1',
  content_hash: 'hash-1',
  content_type: 'text/plain',
  content_data: 'https://analysis.example.com/docs?debug=1',
  source_app: 'Terminal',
  created_at: new Date('2026-03-27T10:00:00Z').getTime(),
  copy_count: 1,
  file_path: null,
  is_favorite: false,
  content_subtype: 'plain_text',
  metadata: null,
  app_bundle_id: null,
  analysis: null,
};

const createEntry = (overrides: Partial<ClipboardEntry>): ClipboardEntry => ({
  ...baseEntry,
  ...overrides,
});

const createAnalysis = (overrides: Partial<EntryAnalysisSnapshot>): EntryAnalysisSnapshot => ({
  contract_version: 1,
  analysis_version: 1,
  status: 'matched',
  subtype: 'plain_text',
  metadata: {
    kind: 'plain_text',
    data: {
      char_count: 38,
      line_count: 1,
    },
  },
  diagnostics: [],
  analyzed_at: new Date('2026-03-27T10:00:00Z').getTime(),
  ...overrides,
});

describe('entryPresentation analysis-first helpers', () => {
  it('优先返回 authoritative analysis subtype 与 diagnostics', () => {
    const entry = createEntry({
      content_subtype: 'json',
      analysis: createAnalysis({
        status: 'fallback',
        subtype: 'plain_text',
        diagnostics: [
          {
            code: 'json_malformed',
            severity: 'error',
            message: 'json parse failed',
          },
        ],
      }),
    });

    expect(getEntryAnalysisSubtype(entry)).toBe('plain_text');
    expect(getEntryAnalysisStatus(entry)).toBe('fallback');
    expect(getEntryAnalysisDiagnostics(entry)).toEqual(
      expect.arrayContaining([expect.objectContaining({ code: 'json_malformed' })])
    );
  });

  it('analysis metadata 会覆盖 legacy metadata 并映射到 presentation shape', () => {
    const entry = createEntry({
      content_subtype: 'json',
      metadata: JSON.stringify({
        url_parts: {
          protocol: 'https',
          host: 'legacy.example.com',
          path: '/legacy',
          query_params: [['mode', 'legacy']],
        },
      }),
      analysis: createAnalysis({
        subtype: 'url',
        metadata: {
          kind: 'url',
          data: {
            protocol: 'https',
            host: 'analysis.example.com',
            path: '/docs',
            query_params: [{ key: 'debug', value: '1' }],
          },
        },
      }),
    });

    expect(getEntryPresentationMetadata(entry)).toEqual({
      url_parts: {
        protocol: 'https',
        host: 'analysis.example.com',
        path: '/docs',
        query_params: [['debug', '1']],
      },
    });

    const summary = buildEntrySemanticSummary(entry);
    expect(summary.headline).toBe('analysis.example.com/docs');
  });

  it('analysis 已存在时不会再退回 legacy metadata', () => {
    const entry = createEntry({
      content_subtype: 'timestamp',
      metadata: JSON.stringify({
        timestamp_formats: {
          unix_ms: 1735699200000,
        },
      }),
      analysis: createAnalysis({
        subtype: 'plain_text',
        metadata: {
          kind: 'plain_text',
          data: {
            char_count: 38,
            line_count: 1,
          },
        },
      }),
    });

    expect(getEntryPresentationMetadata(entry)).toBeNull();
  });

  it('缺少 companion analysis 时仍回退 legacy subtype 与 metadata', () => {
    const entry = createEntry({
      content_subtype: 'color',
      content_data: '#ff5500',
      metadata: JSON.stringify({
        color_formats: {
          hex: '#ff5500',
          rgb: 'rgb(255, 85, 0)',
        },
      }),
      analysis: null,
    });

    expect(getEntryAnalysisSubtype(entry)).toBe('color');
    expect(getEntryPresentationMetadata(entry)).toEqual({
      color_formats: {
        hex: '#ff5500',
        rgb: 'rgb(255, 85, 0)',
      },
    });
  });
});
