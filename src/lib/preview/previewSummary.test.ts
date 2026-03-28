import { describe, expect, it } from 'vitest';
import { ClipboardEntry, EntryAnalysisSnapshot } from '../../types/clipboard';
import { buildPreviewSummary } from './previewSummary';

const baseEntry: ClipboardEntry = {
  id: 'summary-entry-1',
  content_hash: 'summary-hash-1',
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

describe('buildPreviewSummary', () => {
  it('list 与 retrieval 共享同一 semanticType 与 previewIntent', () => {
    const entry = createEntry({
      content_data: 'https://analysis.example.com/docs?debug=1',
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

    const listSummary = buildPreviewSummary(entry, 'list');
    const retrievalSummary = buildPreviewSummary(entry, 'retrieval');

    expect(listSummary.semanticType).toBe('url');
    expect(retrievalSummary.semanticType).toBe('url');
    expect(listSummary.previewIntent).toBe('url_structured');
    expect(retrievalSummary.previewIntent).toBe('url_structured');
    expect(listSummary.headline).toBe(retrievalSummary.headline);
  });

  it('list 保持紧凑，retrieval 只增加 secondary summary 信息量', () => {
    const entry = createEntry({
      content_data: 'const preview = buildPreviewSummary(entry, "list");\nreturn preview;',
      analysis: createAnalysis({
        subtype: 'code',
        metadata: {
          kind: 'code',
          data: {
            detected_language: 'TypeScript',
            line_count: 2,
          },
        },
      }),
    });

    expect(buildPreviewSummary(entry, 'list')).toMatchObject({
      density: 'list',
      semanticType: 'code',
      previewIntent: 'code_workbench',
      headline: 'const preview = buildPreviewSummary(entry, "list");',
      secondarySummary: 'TypeScript',
    });

    expect(buildPreviewSummary(entry, 'retrieval')).toMatchObject({
      density: 'retrieval',
      semanticType: 'code',
      previewIntent: 'code_workbench',
      headline: 'const preview = buildPreviewSummary(entry, "list");',
      secondarySummary: 'TypeScript | 2 lines',
    });
  });

  it('为 URL、JSON、颜色、代码和命令提供明确的 list/retrieval summary contract', () => {
    const urlEntry = createEntry({
      content_data: 'https://analysis.example.com/docs?debug=1',
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

    expect(buildPreviewSummary(urlEntry, 'list')).toMatchObject({
      semanticType: 'url',
      previewIntent: 'url_structured',
      headline: 'analysis.example.com/docs',
      secondarySummary: 'HTTPS',
    });
    expect(buildPreviewSummary(urlEntry, 'retrieval')).toMatchObject({
      semanticType: 'url',
      previewIntent: 'url_structured',
      headline: 'analysis.example.com/docs',
      secondarySummary: 'HTTPS | 1 query param',
    });

    const jsonEntry = createEntry({
      content_data: '{"name":"dance","version":3}',
      analysis: createAnalysis({
        subtype: 'json',
        metadata: {
          kind: 'json',
          data: {
            root_kind: 'object',
            key_count: 2,
          },
        },
      }),
    });

    expect(buildPreviewSummary(jsonEntry, 'list')).toMatchObject({
      semanticType: 'json',
      previewIntent: 'json_structured',
      headline: 'JSON object',
      secondarySummary: '2 keys',
    });
    expect(buildPreviewSummary(jsonEntry, 'retrieval')).toMatchObject({
      semanticType: 'json',
      previewIntent: 'json_structured',
      headline: 'JSON object',
      secondarySummary: '2 keys | 28 chars',
    });

    const colorEntry = createEntry({
      content_data: '#ff5500',
      analysis: createAnalysis({
        subtype: 'color',
        metadata: {
          kind: 'color',
          data: {
            hex: '#ff5500',
            rgb: 'rgb(255, 85, 0)',
            hsl: 'hsl(20, 100%, 50%)',
          },
        },
      }),
    });

    expect(buildPreviewSummary(colorEntry, 'list')).toMatchObject({
      semanticType: 'color',
      previewIntent: 'color_structured',
      headline: '#ff5500',
      secondarySummary: 'rgb(255, 85, 0)',
    });
    expect(buildPreviewSummary(colorEntry, 'retrieval')).toMatchObject({
      semanticType: 'color',
      previewIntent: 'color_structured',
      headline: '#ff5500',
      secondarySummary: 'rgb(255, 85, 0) | hsl(20, 100%, 50%)',
    });

    const commandEntry = createEntry({
      content_data: 'pnpm exec vitest run src/lib/preview/previewSummary.test.ts',
      analysis: createAnalysis({
        subtype: 'command',
        metadata: {
          kind: 'command',
          data: {
            command_name: 'pnpm',
            shell_family: 'bash',
            has_pipeline: false,
            has_sudo_prefix: false,
          },
        },
      }),
    });

    expect(buildPreviewSummary(commandEntry, 'list')).toMatchObject({
      semanticType: 'command',
      previewIntent: 'command_workbench',
      headline: 'pnpm exec vitest run src/lib/preview/previewSummary.test.ts',
      secondarySummary: 'bash',
    });
    expect(buildPreviewSummary(commandEntry, 'retrieval')).toMatchObject({
      semanticType: 'command',
      previewIntent: 'command_workbench',
      headline: 'pnpm exec vitest run src/lib/preview/previewSummary.test.ts',
      secondarySummary: 'bash | command pnpm',
    });
  });

  it('为 plain_text、markdown、email、ip_address、timestamp、base64、image 和 file 提供显式 list/retrieval contract', () => {
    const plainTextEntry = createEntry({
      content_data: 'Clipboard previews stay stable.',
      analysis: createAnalysis({
        subtype: 'plain_text',
        metadata: {
          kind: 'plain_text',
          data: {
            char_count: 31,
            line_count: 1,
          },
        },
      }),
    });

    expect(buildPreviewSummary(plainTextEntry, 'list')).toMatchObject({
      semanticType: 'plain_text',
      previewIntent: 'plain_text_summary',
      headline: 'Clipboard previews stay stable.',
      secondarySummary: '1 line',
    });
    expect(buildPreviewSummary(plainTextEntry, 'retrieval')).toMatchObject({
      semanticType: 'plain_text',
      previewIntent: 'plain_text_summary',
      headline: 'Clipboard previews stay stable.',
      secondarySummary: '1 line | 31 chars',
    });

    const markdownEntry = createEntry({
      content_data: '# Preview Contract\n\n- stable\n- explicit',
      analysis: createAnalysis({
        subtype: 'markdown',
        metadata: {
          kind: 'markdown',
          data: {
            has_heading: true,
            has_list: true,
            has_code_fence: false,
            has_link: false,
          },
        },
      }),
    });

    expect(buildPreviewSummary(markdownEntry, 'list')).toMatchObject({
      semanticType: 'markdown',
      previewIntent: 'markdown_structured',
      headline: 'Preview Contract',
      secondarySummary: 'Heading',
    });
    expect(buildPreviewSummary(markdownEntry, 'retrieval')).toMatchObject({
      semanticType: 'markdown',
      previewIntent: 'markdown_structured',
      headline: 'Preview Contract',
      secondarySummary: 'Heading | List',
    });

    const emailEntry = createEntry({
      content_data: 'preview-team@example.com',
      analysis: createAnalysis({
        subtype: 'email',
        metadata: {
          kind: 'email',
          data: {
            local_part: 'preview-team',
            domain: 'example.com',
          },
        },
      }),
    });

    expect(buildPreviewSummary(emailEntry, 'list')).toMatchObject({
      semanticType: 'email',
      previewIntent: 'email_structured',
      headline: 'preview-team@example.com',
      secondarySummary: 'Domain example.com',
    });
    expect(buildPreviewSummary(emailEntry, 'retrieval')).toMatchObject({
      semanticType: 'email',
      previewIntent: 'email_structured',
      headline: 'preview-team@example.com',
      secondarySummary: 'Domain example.com',
    });

    const ipEntry = createEntry({
      content_data: '10.0.0.15',
      analysis: createAnalysis({
        subtype: 'ip_address',
        metadata: {
          kind: 'ip_address',
          data: {
            version: 'v4',
            is_loopback: false,
            is_private: true,
          },
        },
      }),
    });

    expect(buildPreviewSummary(ipEntry, 'list')).toMatchObject({
      semanticType: 'ip_address',
      previewIntent: 'ip_structured',
      headline: '10.0.0.15',
      secondarySummary: 'IPv4',
    });
    expect(buildPreviewSummary(ipEntry, 'retrieval')).toMatchObject({
      semanticType: 'ip_address',
      previewIntent: 'ip_structured',
      headline: '10.0.0.15',
      secondarySummary: 'IPv4 | Private',
    });

    const timestampEntry = createEntry({
      content_data: '1735689600000',
      analysis: createAnalysis({
        subtype: 'timestamp',
        metadata: {
          kind: 'timestamp',
          data: {
            unix_ms: 1735689600000,
            iso8601: '2025-01-01T00:00:00.000Z',
          },
        },
      }),
    });

    expect(buildPreviewSummary(timestampEntry, 'list')).toMatchObject({
      semanticType: 'timestamp',
      previewIntent: 'timestamp_structured',
      headline: new Date(1735689600000).toLocaleString(),
      secondarySummary: '2025-01-01T00:00:00.000Z',
    });
    expect(buildPreviewSummary(timestampEntry, 'retrieval')).toMatchObject({
      semanticType: 'timestamp',
      previewIntent: 'timestamp_structured',
      headline: new Date(1735689600000).toLocaleString(),
      secondarySummary: '2025-01-01T00:00:00.000Z',
    });

    const base64Entry = createEntry({
      content_data: 'eyJwcmV2aWV3Ijp0cnVlfQ==',
      analysis: createAnalysis({
        subtype: 'base64',
        metadata: {
          kind: 'base64',
          data: {
            encoded_size: 24,
            estimated_original_size: 17,
            content_hint: 'json',
          },
        },
      }),
    });

    expect(buildPreviewSummary(base64Entry, 'list')).toMatchObject({
      semanticType: 'base64',
      previewIntent: 'base64_summary',
      headline: 'Base64 (json)',
      secondarySummary: '24 chars',
    });
    expect(buildPreviewSummary(base64Entry, 'retrieval')).toMatchObject({
      semanticType: 'base64',
      previewIntent: 'base64_summary',
      headline: 'Base64 (json)',
      secondarySummary: '24 chars | 17 bytes decoded',
    });

    const imageEntry = createEntry({
      content_type: 'image/png',
      content_subtype: null,
      content_data: '/tmp/captured-image.png',
      file_path: '/tmp/captured-image.png',
      metadata: JSON.stringify({
        image_metadata: {
          width: 1440,
          height: 900,
          file_size: 2048,
          format: 'png',
        },
      }),
      analysis: null,
    });

    expect(buildPreviewSummary(imageEntry, 'list')).toMatchObject({
      semanticType: 'image',
      previewIntent: 'image_asset',
      headline: 'captured-image.png',
      secondarySummary: 'PNG',
    });
    expect(buildPreviewSummary(imageEntry, 'retrieval')).toMatchObject({
      semanticType: 'image',
      previewIntent: 'image_asset',
      headline: 'captured-image.png',
      secondarySummary: 'PNG | 1440x900',
    });

    const fileEntry = createEntry({
      content_type: 'file/path',
      content_subtype: null,
      content_data: '/tmp/preview-contract.sql',
      file_path: '/tmp/preview-contract.sql',
      metadata: JSON.stringify({
        file_metadata: {
          name: 'preview-contract.sql',
          extension: 'sql',
          mime: 'text/x-sql',
        },
      }),
      analysis: null,
    });

    expect(buildPreviewSummary(fileEntry, 'list')).toMatchObject({
      semanticType: 'file',
      previewIntent: 'file_asset',
      headline: 'preview-contract.sql',
      secondarySummary: 'SQL',
    });
    expect(buildPreviewSummary(fileEntry, 'retrieval')).toMatchObject({
      semanticType: 'file',
      previewIntent: 'file_asset',
      headline: 'preview-contract.sql',
      secondarySummary: 'SQL | text/x-sql',
    });
  });
});
