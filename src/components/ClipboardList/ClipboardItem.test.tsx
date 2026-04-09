import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useClipboardStore } from '../../stores/clipboardStore';
import type { ClipboardEntry, EntryAnalysisSnapshot } from '../../types/clipboard';
import * as previewSummaryModule from '../../lib/preview/previewSummary';
import { buildPreviewSummary } from '../../lib/preview/previewSummary';
import { ClipboardItem } from './ClipboardItem';

vi.mock('../../stores/clipboardStore', () => ({
  useClipboardStore: vi.fn(),
}));

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, options?: { count?: number }) => {
      if (key === 'clipboard:actions.copiedTimes') {
        return `Copied ${options?.count ?? 0} times`;
      }

      return key;
    },
  }),
}));

const mockedUseClipboardStore = vi.mocked(useClipboardStore);

const getImageUrl = vi.fn(() => new Promise<string>(() => {}));
const getAppIcon = vi.fn(() => new Promise<string>(() => {}));

const baseEntry: ClipboardEntry = {
  id: 'clipboard-item-entry',
  content_hash: 'clipboard-item-hash',
  content_type: 'text/plain',
  content_data: 'placeholder content',
  source_app: 'Terminal',
  created_at: new Date('2026-03-28T10:00:00Z').getTime(),
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
      char_count: 19,
      line_count: 1,
    },
  },
  diagnostics: [],
  analyzed_at: new Date('2026-03-28T10:00:00Z').getTime(),
  ...overrides,
});

const createStoreState = () => ({
  toggleFavorite: vi.fn(),
  deleteEntry: vi.fn(),
  copyToClipboard: vi.fn(),
  getImageUrl,
  pasteSelectedEntry: vi.fn(),
  getAppIcon,
});

const renderItem = (entry: ClipboardEntry) =>
  render(<ClipboardItem entry={entry} onClick={vi.fn()} showNumber={false} />);

describe('ClipboardItem', () => {
  beforeEach(() => {
    getImageUrl.mockClear();
    getAppIcon.mockClear();
    mockedUseClipboardStore.mockReturnValue(createStoreState());
  });

  it.each([
    [
      'JSON',
      createEntry({
        content_data: '{"name":"dance","phase":"03","status":"ready"}',
        analysis: createAnalysis({
          subtype: 'json',
          metadata: {
            kind: 'json',
            data: {
              root_kind: 'object',
              key_count: 3,
            },
          },
        }),
      }),
    ],
    [
      'URL',
      createEntry({
        content_data: 'https://analysis.example.com/docs/install?debug=1',
        analysis: createAnalysis({
          subtype: 'url',
          metadata: {
            kind: 'url',
            data: {
              protocol: 'https',
              host: 'analysis.example.com',
              path: '/docs/install',
              query_params: [{ key: 'debug', value: '1' }],
            },
          },
        }),
      }),
    ],
    [
      'Color',
      createEntry({
        content_data: '#0EA5E9',
        analysis: createAnalysis({
          subtype: 'color',
          metadata: {
            kind: 'color',
            data: {
              hex: '#0EA5E9',
              rgb: 'rgb(14, 165, 233)',
              hsl: 'hsl(199, 89%, 48%)',
            },
          },
        }),
      }),
    ],
    [
      'Code',
      createEntry({
        content_data:
          'const executePhasePlan = () => runUnifiedClipboardSummaryShell();\nreturn executePhasePlan();',
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
      }),
    ],
    [
      'Command',
      createEntry({
        content_data:
          'pnpm exec vitest run src/components/ClipboardList/ClipboardItem.test.tsx --reporter=dot',
        analysis: createAnalysis({
          subtype: 'command',
          metadata: {
            kind: 'command',
            data: {
              command_name: 'pnpm',
              shell_family: 'zsh',
              has_pipeline: false,
              has_sudo_prefix: false,
            },
          },
        }),
      }),
    ],
  ])('为 %s 条目渲染统一两层 summary shell', (_label, entry) => {
    const summary = buildPreviewSummary(entry, 'list');
    const { container, unmount } = renderItem(entry);

    const card = container.querySelector('.group');
    expect(card).toHaveClass('min-h-[100px]');

    const headline = screen.getByText(summary.headline);
    expect(headline).toHaveClass('truncate');

    const secondarySummary = screen.getByText(summary.secondarySummary);
    expect(secondarySummary).toHaveClass('max-h-8', 'overflow-hidden');

    expect(getImageUrl).not.toHaveBeenCalled();

    unmount();
  });

  it('对长内容保持稳定的最小高度，并为 headline/secondary summary 施加稳定截断样式', () => {
    const entry = createEntry({
      content_data:
        'const summaryHeadlineShouldStayOnOneLine = "summary shell should stay compact even when the clipboard item contains a deliberately verbose code sample for testing";\nconst secondarySummaryShouldClampToTwoLines = "Phase 03 keeps list density stable and never lets one entry expand the row height unexpectedly";',
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
    const summary = buildPreviewSummary(entry, 'list');
    const { container } = renderItem(entry);

    expect(container.querySelector('.group')).toHaveClass('min-h-[100px]');
    expect(screen.getByText(summary.headline)).toHaveClass('truncate');
    expect(screen.getByText(summary.secondarySummary)).toHaveClass('max-h-8', 'overflow-hidden');
  });

  it('Image 条目在列表中展示缩略图、分辨率和大小，不再展示文件标题', async () => {
    getImageUrl.mockResolvedValueOnce('tauri://list-image-preview');
    const entry = createEntry({
      content_type: 'image/png',
      content_data: null,
      file_path: '/tmp/screenshots/unified-preview.png',
      metadata: JSON.stringify({
        image_metadata: {
          format: 'png',
          width: 1440,
          height: 900,
          file_size: 2048,
        },
      }),
    });

    renderItem(entry);

    expect(await screen.findByAltText('clipboard:contentTypes.image')).toHaveAttribute(
      'src',
      'tauri://list-image-preview'
    );
    expect(screen.getByText('1440 x 900')).toBeInTheDocument();
    expect(screen.getByText('2 KB')).toBeInTheDocument();
    expect(screen.queryByText('unified-preview.png')).not.toBeInTheDocument();
    expect(getImageUrl).toHaveBeenCalledWith('/tmp/screenshots/unified-preview.png');
  });

  it('当 summary contract 意外为空时，列表项会回退到原始文本预览而不是渲染空白区域', () => {
    const summarySpy = vi.spyOn(previewSummaryModule, 'buildPreviewSummary').mockReturnValue({
      density: 'list',
      semanticType: 'plain_text',
      previewIntent: 'plain_text_summary',
      headline: '',
      secondarySummary: '',
    });

    renderItem(
      createEntry({
        content_data: 'ANALYSIS should still render as visible fallback text',
      })
    );

    expect(
      screen.getAllByText('ANALYSIS should still render as visible fallback text').length
    ).toBeGreaterThan(0);

    summarySpy.mockRestore();
  });

  it('展示快捷数字角标时保持圆形并让数字居中', () => {
    render(
      <ClipboardItem entry={createEntry({ content_data: 'quick paste' })} showNumber number={7} />
    );

    expect(screen.getByText('7')).toHaveClass(
      'h-5',
      'w-5',
      'justify-center',
      'text-center',
      'leading-none'
    );
  });
});
