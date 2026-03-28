import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useClipboardStore } from '../../../stores/clipboardStore';
import { ColorRenderer } from './ColorRenderer';

const mockedUseClipboardStore = vi.mocked(useClipboardStore);

vi.mock('../../../stores/clipboardStore', () => ({
  useClipboardStore: vi.fn(),
}));

describe('ColorRenderer', () => {
  beforeEach(() => {
    mockedUseClipboardStore.mockReturnValue({
      copyToClipboard: vi.fn().mockResolvedValue(undefined),
    } as ReturnType<typeof useClipboardStore>);

    Object.defineProperty(window.navigator, 'clipboard', {
      configurable: true,
      value: {
        writeText: vi.fn(),
      },
    });
  });

  it('优先使用 metadata 中的 color_formats 并按 HEX/RGB/RGBA/HSL 顺序展示', () => {
    render(
      <ColorRenderer
        content="rgb(1, 2, 3)"
        metadata={JSON.stringify({
          metadata: 'preferred',
          color_formats: {
            hex: '#1E293B',
            rgb: 'rgb(30, 41, 59)',
            rgba: 'rgba(30, 41, 59, 0.85)',
            hsl: 'hsl(216, 33%, 17%)',
          },
        })}
      />
    );

    expect(screen.getByTestId('color-swatch')).toBeInTheDocument();
    expect(screen.getByText('RGB 值:')).toBeInTheDocument();

    const rows = screen.getAllByTestId('color-format-row');
    expect(rows.map((row) => row.getAttribute('data-format-label'))).toEqual([
      'HEX',
      'RGB',
      'RGBA',
      'HSL',
    ]);

    expect(screen.getByTestId('color-format-hex')).toHaveTextContent('#1E293B');
    expect(screen.getByTestId('color-format-rgb')).toHaveTextContent('rgb(30, 41, 59)');
    expect(screen.getByTestId('color-format-rgba')).toHaveTextContent('rgba(30, 41, 59, 0.85)');
    expect(screen.getByTestId('color-format-hsl')).toHaveTextContent('hsl(216, 33%, 17%)');
  });

  it('metadata 缺失时回退到本地解析并显示四种标准格式', () => {
    render(<ColorRenderer content="#ff8800" metadata={null} />);

    expect(screen.getByTestId('color-format-hex')).toHaveTextContent(/#ff8800/i);
    expect(screen.getByTestId('color-format-rgb')).toHaveTextContent('rgb(255, 136, 0)');
    expect(screen.getByTestId('color-format-rgba')).toHaveTextContent('rgba(255, 136, 0, 1)');
    expect(screen.getByTestId('color-format-hsl')).toHaveTextContent('hsl(32, 100%, 50%)');
  });

  it('复制原始值和各格式都统一走 backend copy contract', () => {
    const copyToClipboard = vi.fn().mockResolvedValue(undefined);
    mockedUseClipboardStore.mockReturnValue({
      copyToClipboard,
    } as ReturnType<typeof useClipboardStore>);

    render(
      <ColorRenderer
        content="#1e293b"
        metadata={JSON.stringify({
          color_formats: {
            hex: '#1E293B',
            rgb: 'rgb(30, 41, 59)',
            rgba: 'rgba(30, 41, 59, 1)',
            hsl: 'hsl(216, 33%, 17%)',
          },
        })}
      />
    );

    fireEvent.click(screen.getByRole('button', { name: '复制原始值' }));
    fireEvent.click(screen.getByRole('button', { name: '复制 HEX' }));
    fireEvent.click(screen.getByRole('button', { name: '复制 RGB' }));
    fireEvent.click(screen.getByRole('button', { name: '复制 RGBA' }));
    fireEvent.click(screen.getByRole('button', { name: '复制 HSL' }));

    expect(copyToClipboard).toHaveBeenNthCalledWith(1, '#1e293b');
    expect(copyToClipboard).toHaveBeenNthCalledWith(2, '#1E293B');
    expect(copyToClipboard).toHaveBeenNthCalledWith(3, 'rgb(30, 41, 59)');
    expect(copyToClipboard).toHaveBeenNthCalledWith(4, 'rgba(30, 41, 59, 1)');
    expect(copyToClipboard).toHaveBeenNthCalledWith(5, 'hsl(216, 33%, 17%)');
    expect(window.navigator.clipboard.writeText).not.toHaveBeenCalled();
  });
});
