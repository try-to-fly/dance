import { render, screen } from '@testing-library/react';
import { act } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { SearchBar } from './SearchBar';
import { FOCUS_SEARCH_INPUT_EVENT } from '../../lib/appEvents';
import { useClipboardStore } from '../../stores/clipboardStore';

vi.mock('../../stores/clipboardStore', () => ({
  useClipboardStore: vi.fn(),
}));

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, options?: { defaultValue?: string }) => options?.defaultValue ?? key,
  }),
}));

vi.mock('../../services/analytics', () => ({
  ANALYTICS_EVENTS: {
    SEARCH_PERFORMED: 'search_performed',
  },
  analytics: {
    track: vi.fn(),
  },
}));

const mockedUseClipboardStore = vi.mocked(useClipboardStore);

describe('SearchBar', () => {
  beforeEach(() => {
    mockedUseClipboardStore.mockReturnValue({
      searchTerm: 'existing query',
      setSearchTerm: vi.fn(),
      selectedType: 'all',
      selectedSourceApp: 'all',
      favoritesOnly: false,
      loading: false,
    });
  });

  it('搜索输入框关闭自动大写、拼写检查和自动纠正', () => {
    render(<SearchBar compact />);

    const input = screen.getByRole('searchbox');
    expect(input).toHaveAttribute('autocapitalize', 'none');
    expect(input).toHaveAttribute('autocorrect', 'off');
    expect(input).toHaveAttribute('autocomplete', 'off');
    expect(input).toHaveAttribute('spellcheck', 'false');
  });

  it('收到聚焦事件后会激活搜索输入框并选中文本', () => {
    const requestAnimationFrameSpy = vi
      .spyOn(window, 'requestAnimationFrame')
      .mockImplementation((callback: FrameRequestCallback) => {
        callback(0);
        return 1;
      });

    render(<SearchBar compact />);

    act(() => {
      window.dispatchEvent(new Event(FOCUS_SEARCH_INPUT_EVENT));
    });

    const input = screen.getByRole('searchbox');
    expect(input).toHaveFocus();
    expect((input as HTMLInputElement).selectionStart).toBe(0);
    expect((input as HTMLInputElement).selectionEnd).toBe('existing query'.length);

    requestAnimationFrameSpy.mockRestore();
  });
});
