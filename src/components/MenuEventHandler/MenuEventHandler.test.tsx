import { render, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { MenuEventHandler } from './MenuEventHandler';
import { FOCUS_SEARCH_INPUT_EVENT } from '../../lib/appEvents';
import { useConfigStore } from '../../stores/configStore';
import { useClipboardStore } from '../../stores/clipboardStore';
import { useAiStore } from '../../stores/aiStore';
import { listen } from '@tauri-apps/api/event';

vi.mock('../../stores/configStore', () => ({
  useConfigStore: vi.fn(),
}));

vi.mock('../../stores/clipboardStore', () => ({
  useClipboardStore: vi.fn(),
}));

vi.mock('../../stores/aiStore', () => ({
  useAiStore: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

const mockedUseConfigStore = vi.mocked(useConfigStore);
const mockedUseClipboardStore = vi.mocked(useClipboardStore);
const mockedUseAiStore = vi.mocked(useAiStore);
const mockedListen = vi.mocked(listen);
let setShowPreferencesSpy: ReturnType<typeof vi.fn>;
let resetRetrievalFiltersSpy: ReturnType<typeof vi.fn>;
let closeDialogSpy: ReturnType<typeof vi.fn>;

describe('MenuEventHandler', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    setShowPreferencesSpy = vi.fn();
    resetRetrievalFiltersSpy = vi.fn();
    closeDialogSpy = vi.fn();

    mockedUseConfigStore.mockImplementation(((
      selector: (state: { setShowPreferences: typeof setShowPreferencesSpy }) => unknown
    ) => selector({ setShowPreferences: setShowPreferencesSpy })) as never);
    mockedUseClipboardStore.mockImplementation(((
      selector: (state: { resetRetrievalFilters: typeof resetRetrievalFiltersSpy }) => unknown
    ) => selector({ resetRetrievalFilters: resetRetrievalFiltersSpy })) as never);
    mockedUseAiStore.mockImplementation(((
      selector: (state: { closeDialog: typeof closeDialogSpy }) => unknown
    ) => selector({ closeDialog: closeDialogSpy })) as never);
    mockedListen.mockImplementation(async () => (() => {}) as () => void);
  });

  it('全局快捷键事件会重置搜索态并派发搜索框聚焦事件', async () => {
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

    expect(resetRetrievalFiltersSpy).toHaveBeenCalledTimes(1);
    expect(closeDialogSpy).toHaveBeenCalledTimes(1);
    expect(windowFocusSpy).toHaveBeenCalledTimes(1);
    expect(scrollSpy).toHaveBeenCalledWith(0, 0);
    expect(dispatchSpy).toHaveBeenCalledWith(
      expect.objectContaining({ type: FOCUS_SEARCH_INPUT_EVENT })
    );
  });
});
