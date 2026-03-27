import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { JsonRenderer } from './JsonRenderer';

const mockedJsonRendererDeps = vi.hoisted(() => ({
  invoke: vi.fn().mockResolvedValue(undefined),
  writeText: vi.fn().mockResolvedValue(undefined),
  defineMonacoThemes: vi.fn(),
  monacoLoaderConfig: vi.fn(),
  monacoRender: vi.fn(
    ({
      beforeMount,
      height,
      language,
      value,
    }: {
      beforeMount?: (instance: unknown) => void;
      height: string;
      language: string;
      value: string;
    }) => {
      beforeMount?.({});
      return (
        <div data-height={height} data-language={language} data-testid="json-monaco-editor">
          {value}
        </div>
      );
    }
  ),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockedJsonRendererDeps.invoke,
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const dictionary: Record<string, string> = {
        'codeEditor.json': 'JSON',
        'codeEditor.copy': '复制',
        'codeEditor.copied': '已复制',
        'jsonView.treeView': '树形视图',
        'jsonView.codeView': '代码视图',
        'jsonView.switchToCode': '切换到代码视图',
        'jsonView.switchToTree': '切换到树形视图',
        'jsonView.invalidJson': '无效的 JSON 格式',
      };

      return dictionary[key] ?? key;
    },
  }),
}));

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({
  writeText: mockedJsonRendererDeps.writeText,
}));

vi.mock('../../../hooks/useResolvedTheme', () => ({
  useResolvedTheme: () => 'light',
}));

vi.mock('../../../utils/monacoTheme', () => ({
  defineMonacoThemes: mockedJsonRendererDeps.defineMonacoThemes,
}));

vi.mock('react-json-view-lite', () => ({
  JsonView: ({ data }: { data: unknown }) => (
    <div data-testid="json-tree-view">{JSON.stringify(data)}</div>
  ),
  darkStyles: {},
  defaultStyles: {},
}));

vi.mock('@monaco-editor/react', () => ({
  loader: {
    config: mockedJsonRendererDeps.monacoLoaderConfig,
  },
  default: mockedJsonRendererDeps.monacoRender,
}));

vi.mock('monaco-editor', () => ({}));

describe('JsonRenderer', () => {
  it('JSON 内容默认进入结构化树视图', () => {
    render(<JsonRenderer content='{"profile":{"name":"dance","id":1}}' />);

    expect(screen.getByText('JSON')).toBeInTheDocument();
    expect(screen.getByText('树形视图')).toBeInTheDocument();
    expect(screen.getByTestId('json-tree-view')).toHaveTextContent('"name":"dance"');
    expect(screen.queryByTestId('json-monaco-editor')).not.toBeInTheDocument();
  });

  it('切换到代码视图后渲染 JSON 代码编辑器', () => {
    render(<JsonRenderer content='{"enabled":true}' />);

    fireEvent.click(screen.getByTitle('切换到代码视图'));

    expect(screen.getByText('代码视图')).toBeInTheDocument();
    expect(screen.getByTestId('json-monaco-editor')).toHaveAttribute('data-language', 'json');
    expect(screen.getByTestId('json-monaco-editor')).toHaveAttribute('data-height', '100%');
    expect(mockedJsonRendererDeps.defineMonacoThemes).toHaveBeenCalledTimes(1);
  });

  it('复制按钮走 backend copy_to_clipboard 合同', () => {
    render(<JsonRenderer content='{"enabled":true}' />);

    fireEvent.click(screen.getByRole('button', { name: '复制' }));

    expect(mockedJsonRendererDeps.invoke).toHaveBeenCalledWith('copy_to_clipboard', {
      content: '{\n  "enabled": true\n}',
    });
    expect(mockedJsonRendererDeps.writeText).not.toHaveBeenCalled();
  });
});
