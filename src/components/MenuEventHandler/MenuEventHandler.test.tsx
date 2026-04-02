import { render, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { MenuEventHandler } from './MenuEventHandler';
import { FOCUS_SEARCH_INPUT_EVENT } from '../../lib/appEvents';
import { useConfigStore } from '../../stores/configStore';
import { listen } from '@tauri-apps/api/event';

vi.mock('../../stores/configStore', () => ({
  useConfigStore: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

const mockedUseConfigStore = vi.mocked(useConfigStore);
const mockedListen = vi.mocked(listen);

describe('MenuEventHandler', () => {
  beforeEach(() => {
    mockedUseConfigStore.mockReturnValue({
      setShowPreferences: vi.fn(),
    } as ReturnType<typeof useConfigStore>);
    mockedListen.mockImplementation(async () => (() => {}) as () => void);
  });

  it('全局快捷键事件会派发搜索框聚焦事件', async () => {
    const windowFocusSpy = vi.spyOn(window, 'focus').mockImplementation(() => {});
    const scrollSpy = vi.spyOn(window, 'scrollTo').mockImplementation(() => {});
    const dispatchSpy = vi.spyOn(window, 'dispatchEvent');

    render(<MenuEventHandler />);

    await waitFor(() => {
      expect(mockedListen).toHaveBeenCalled();
    });

    const globalShortcutListenerCall = mockedListen.mock.calls.find(
      ([eventName]) => eventName === 'global-shortcut'
    );

    expect(globalShortcutListenerCall).toBeDefined();

    const handler = globalShortcutListenerCall?.[1];
    await handler?.({ payload: null } as never);

    expect(windowFocusSpy).toHaveBeenCalledTimes(1);
    expect(scrollSpy).toHaveBeenCalledWith(0, 0);
    expect(dispatchSpy).toHaveBeenCalledWith(
      expect.objectContaining({ type: FOCUS_SEARCH_INPUT_EVENT })
    );
  });
});
