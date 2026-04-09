import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { UnifiedTextRenderer } from './UnifiedTextRenderer';

const mockedMonaco = vi.hoisted(() => ({
  invoke: vi.fn().mockResolvedValue(undefined),
  loaderConfig: vi.fn(),
  render: vi.fn(
    ({
      beforeMount,
      height,
      language,
      onChange,
      theme,
      value,
    }: {
      beforeMount?: (instance: unknown) => void;
      height: string;
      language: string;
      onChange?: (value?: string) => void;
      theme: string;
      value: string;
    }) => {
      beforeMount?.({});
      return (
        <div
          data-height={height}
          data-language={language}
          data-testid="monaco-editor"
          data-theme={theme}
        >
          {value}
          <button onClick={() => onChange?.('editedValue')} type="button">
            mutate-editor
          </button>
        </div>
      );
    }
  ),
  writeText: vi.fn().mockResolvedValue(undefined),
  defineMonacoThemes: vi.fn(),
  generateCodeSnapshotDataUrl: vi.fn().mockResolvedValue('data:image/png;base64,snapshot'),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockedMonaco.invoke,
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const dictionary: Record<string, string> = {
        'codeEditor.code': '代码',
        'codeEditor.command': '命令',
        'codeEditor.copy': '复制',
        'codeEditor.copied': '已复制',
        'codeEditor.snapshot': '代码截图',
        'codeEditor.snapshotting': '生成中...',
        'codeEditor.snapshotCopied': '截图已复制',
        'codeEditor.text': '文本',
        'codeEditor.markdown': 'Markdown',
        'codeEditor.json': 'JSON',
        'codeEditor.copyFailed': '复制失败',
        'codeEditor.snapshotFailed': '代码截图失败',
      };

      return dictionary[key] ?? key;
    },
  }),
}));

vi.mock('@monaco-editor/react', () => ({
  loader: {
    config: mockedMonaco.loaderConfig,
  },
  default: mockedMonaco.render,
}));

vi.mock('monaco-editor', () => ({}));

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({
  writeText: mockedMonaco.writeText,
}));

vi.mock('../../../hooks/useResolvedTheme', () => ({
  useResolvedTheme: () => 'light',
}));

vi.mock('../../../utils/monacoTheme', () => ({
  defineMonacoThemes: mockedMonaco.defineMonacoThemes,
}));

vi.mock('../../../lib/codeSnapshot', () => ({
  generateCodeSnapshotDataUrl: mockedMonaco.generateCodeSnapshotDataUrl,
}));

describe('UnifiedTextRenderer', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockedMonaco.generateCodeSnapshotDataUrl.mockResolvedValue('data:image/png;base64,snapshot');
  });

  // PREV-04 read-only wording is overridden by D-14..D-17 for local workbench behavior.
  it('为代码内容展示检测出的语言，并保留可伸展的编辑器高度', () => {
    render(
      <UnifiedTextRenderer
        content="const answer = 42;"
        contentSubType="code"
        metadata='{"detected_language":"typescript"}'
      />
    );

    expect(screen.getByText('代码')).toBeInTheDocument();
    expect(screen.getByText('typescript')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '代码截图' })).toBeInTheDocument();
    expect(screen.getByTestId('monaco-editor')).toHaveAttribute('data-language', 'typescript');
    expect(screen.getByTestId('monaco-editor')).toHaveAttribute('data-height', '100%');
    expect(document.getElementById('text-renderer')).toHaveClass(
      'flex',
      'min-h-[320px]',
      'min-w-0',
      'flex-1',
      'flex-col',
      'overflow-hidden'
    );
    expect(document.getElementById('text-renderer-content')).toHaveClass(
      'flex',
      'min-h-0',
      'flex-1',
      'flex-col',
      'overflow-hidden'
    );
    expect(document.getElementById('text-renderer-editor-container')).toHaveClass(
      'flex',
      'min-h-0',
      'flex-1',
      'border-t'
    );
    expect(mockedMonaco.defineMonacoThemes).toHaveBeenCalledTimes(1);
  });

  it('为命令内容使用 shell 语言并支持复制正文', async () => {
    render(<UnifiedTextRenderer content="pnpm lint" contentSubType="command" />);

    expect(screen.getByText('命令')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '代码截图' })).toBeInTheDocument();
    expect(screen.getByTestId('monaco-editor')).toHaveAttribute('data-language', 'shell');

    fireEvent.click(screen.getByRole('button', { name: '复制' }));

    expect(mockedMonaco.invoke).toHaveBeenCalledWith('copy_to_clipboard', {
      content: 'pnpm lint',
    });
    expect(mockedMonaco.writeText).not.toHaveBeenCalled();
  });

  it('代码截图按钮会生成高亮卡片并复制图片到剪贴板', async () => {
    render(
      <UnifiedTextRenderer
        content="const answer = 42;"
        contentSubType="code"
        metadata='{"detected_language":"typescript"}'
      />
    );

    fireEvent.click(screen.getByRole('button', { name: '代码截图' }));

    await waitFor(() => {
      expect(mockedMonaco.generateCodeSnapshotDataUrl).toHaveBeenCalledWith({
        content: 'const answer = 42;',
        language: 'typescript',
        theme: 'light',
        title: '代码',
        showLineNumbers: true,
      });
      expect(mockedMonaco.invoke).toHaveBeenCalledWith('copy_converted_image', {
        base64Data: 'data:image/png;base64,snapshot',
        skipRecording: true,
      });
    });
  });

  it('命令截图按钮会按 shell 语言生成图片', async () => {
    render(<UnifiedTextRenderer content="pnpm lint" contentSubType="command" />);

    fireEvent.click(screen.getByRole('button', { name: '代码截图' }));

    await waitFor(() => {
      expect(mockedMonaco.generateCodeSnapshotDataUrl).toHaveBeenCalledWith({
        content: 'pnpm lint',
        language: 'shell',
        theme: 'light',
        title: '命令',
        showLineNumbers: true,
      });
      expect(mockedMonaco.invoke).toHaveBeenCalledWith('copy_converted_image', {
        base64Data: 'data:image/png;base64,snapshot',
        skipRecording: true,
      });
    });
  });

  it('D-15 / D-16: sessionKey 切换时重置本地 workbench，并通过 onContentChange 上报当前 buffer', () => {
    const onContentChange = vi.fn();
    const { rerender } = render(
      <UnifiedTextRenderer
        content="echo same"
        contentSubType="command"
        sessionKey="entry-a"
        onContentChange={onContentChange}
      />
    );

    expect(onContentChange).toHaveBeenCalledWith('echo same');

    fireEvent.click(screen.getByRole('button', { name: 'mutate-editor' }));

    expect(screen.getByTestId('monaco-editor')).toHaveTextContent('editedValue');
    expect(onContentChange).toHaveBeenLastCalledWith('editedValue');

    rerender(
      <UnifiedTextRenderer
        content="echo same"
        contentSubType="command"
        sessionKey="entry-b"
        onContentChange={onContentChange}
      />
    );

    expect(screen.getByTestId('monaco-editor')).toHaveTextContent('echo same');
    expect(onContentChange).toHaveBeenLastCalledWith('echo same');
  });
});
