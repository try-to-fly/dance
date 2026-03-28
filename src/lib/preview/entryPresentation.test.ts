import { describe, expect, it } from 'vitest';
import {
  buildEntrySemanticSummary,
  buildSemanticPreviewModel,
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

describe('buildSemanticPreviewModel', () => {
  it('为 JSON、URL、颜色、代码和命令返回稳定的 semantic type 与 preview intent', () => {
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

    expect(buildSemanticPreviewModel(jsonEntry)).toMatchObject({
      semanticType: 'json',
      previewIntent: 'json_structured',
      headline: 'JSON object',
      secondarySummary: '2 keys | 28 chars',
      supportsRawView: true,
      usesWorkbench: false,
    });

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

    expect(buildSemanticPreviewModel(urlEntry)).toMatchObject({
      semanticType: 'url',
      previewIntent: 'url_structured',
      headline: 'analysis.example.com/docs',
      secondarySummary: 'HTTPS | 1 query param',
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

    expect(buildSemanticPreviewModel(colorEntry)).toMatchObject({
      semanticType: 'color',
      previewIntent: 'color_structured',
      headline: '#ff5500',
      secondarySummary: 'rgb(255, 85, 0) | hsl(20, 100%, 50%)',
      supportsRawView: true,
      usesWorkbench: false,
    });

    const codeEntry = createEntry({
      content_data: 'const preview = buildSemanticPreviewModel(entry);\nreturn preview;',
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

    expect(buildSemanticPreviewModel(codeEntry)).toMatchObject({
      semanticType: 'code',
      previewIntent: 'code_workbench',
      headline: 'const preview = buildSemanticPreviewModel(entry);',
      secondarySummary: 'TypeScript | 2 lines',
      supportsRawView: true,
      usesWorkbench: true,
    });

    const commandEntry = createEntry({
      content_data: 'pnpm exec vitest run src/lib/preview/entryPresentation.test.ts',
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

    expect(buildSemanticPreviewModel(commandEntry)).toMatchObject({
      semanticType: 'command',
      previewIntent: 'command_workbench',
      headline: 'pnpm exec vitest run src/lib/preview/entryPresentation.test.ts',
      secondarySummary: 'bash | command pnpm',
      supportsRawView: true,
      usesWorkbench: true,
    });
  });

  it('URL semantic model 只依赖本地 url_parts 与 raw URL，不依赖任何远端 resolved 信息', () => {
    const entry = createEntry({
      content_data: 'https://analysis.example.com/docs/reference?debug=1&mode=full',
      metadata: JSON.stringify({
        resolved_preview_summary: {
          title: 'Remote page title',
          kind: 'json',
        },
      }),
      analysis: createAnalysis({
        subtype: 'url',
        metadata: {
          kind: 'url',
          data: {
            protocol: 'https',
            host: 'analysis.example.com',
            path: '/docs/reference',
            query_params: [
              { key: 'debug', value: '1' },
              { key: 'mode', value: 'full' },
            ],
          },
        },
      }),
    });

    const model = buildSemanticPreviewModel(entry);
    expect(model).toMatchObject({
      semanticType: 'url',
      previewIntent: 'url_structured',
      headline: 'analysis.example.com/docs/reference',
      secondarySummary: 'HTTPS | 2 query params',
    });
    expect(model.secondarySummary).not.toContain('Remote page title');
  });

  it('代码和命令 semantic model 显式标记 usesWorkbench 与 supportsRawView', () => {
    const codeModel = buildSemanticPreviewModel(
      createEntry({
        content_data: 'console.log("preview");',
        analysis: createAnalysis({
          subtype: 'code',
          metadata: {
            kind: 'code',
            data: {
              detected_language: 'JavaScript',
              line_count: 1,
            },
          },
        }),
      })
    );

    const commandModel = buildSemanticPreviewModel(
      createEntry({
        content_data: 'git status --short',
        analysis: createAnalysis({
          subtype: 'command',
          metadata: {
            kind: 'command',
            data: {
              command_name: 'git',
              shell_family: 'zsh',
              has_pipeline: false,
              has_sudo_prefix: false,
            },
          },
        }),
      })
    );

    expect(codeModel.supportsRawView).toBe(true);
    expect(codeModel.usesWorkbench).toBe(true);
    expect(commandModel.supportsRawView).toBe(true);
    expect(commandModel.usesWorkbench).toBe(true);
  });

  it('为 plain_text、markdown、email、ip_address、timestamp、base64 以及顶层 image/file 提供显式 fallback contract', () => {
    const plainTextModel = buildSemanticPreviewModel(
      createEntry({
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
      })
    );

    expect(plainTextModel).toMatchObject({
      semanticType: 'plain_text',
      previewIntent: 'plain_text_summary',
      headline: 'Clipboard previews stay stable.',
      secondarySummary: '1 line | 31 chars',
    });

    const markdownModel = buildSemanticPreviewModel(
      createEntry({
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
      })
    );

    expect(markdownModel).toMatchObject({
      semanticType: 'markdown',
      previewIntent: 'markdown_structured',
      headline: 'Preview Contract',
      secondarySummary: 'Heading | List',
    });

    const emailModel = buildSemanticPreviewModel(
      createEntry({
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
      })
    );

    expect(emailModel).toMatchObject({
      semanticType: 'email',
      previewIntent: 'email_structured',
      headline: 'preview-team@example.com',
      secondarySummary: 'Domain example.com',
    });

    const ipModel = buildSemanticPreviewModel(
      createEntry({
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
      })
    );

    expect(ipModel).toMatchObject({
      semanticType: 'ip_address',
      previewIntent: 'ip_structured',
      headline: '10.0.0.15',
      secondarySummary: 'IPv4 | Private',
    });

    const timestampModel = buildSemanticPreviewModel(
      createEntry({
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
      })
    );

    expect(timestampModel).toMatchObject({
      semanticType: 'timestamp',
      previewIntent: 'timestamp_structured',
      headline: new Date(1735689600000).toLocaleString(),
      secondarySummary: '2025-01-01T00:00:00.000Z',
    });

    const base64Model = buildSemanticPreviewModel(
      createEntry({
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
      })
    );

    expect(base64Model).toMatchObject({
      semanticType: 'base64',
      previewIntent: 'base64_summary',
      headline: 'Base64 (json)',
      secondarySummary: '24 chars | 17 bytes decoded',
    });

    const imageModel = buildSemanticPreviewModel(
      createEntry({
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
      })
    );

    expect(imageModel).toMatchObject({
      semanticType: 'image',
      previewIntent: 'image_asset',
      headline: 'captured-image.png',
      secondarySummary: 'PNG | 1440x900',
    });

    const fileModel = buildSemanticPreviewModel(
      createEntry({
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
      })
    );

    expect(fileModel).toMatchObject({
      semanticType: 'file',
      previewIntent: 'file_asset',
      headline: 'preview-contract.sql',
      secondarySummary: 'SQL | text/x-sql',
    });
  });
});
