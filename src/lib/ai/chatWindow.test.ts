import { beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import {
  AI_CHAT_WINDOW_LABEL,
  AI_CHAT_WINDOW_NAVIGATE_EVENT,
  openAiChatWindow,
} from './chatWindow';

const mockedInvoke = vi.mocked(invoke);

const { getByLabelMock, createdWindowRecords, MockWebviewWindow } = vi.hoisted(() => {
  const getByLabelMock = vi.fn();
  const createdWindowRecords: Array<{ label: string; options: Record<string, unknown> }> = [];

  class MockWebviewWindow {
    static getByLabel = getByLabelMock;

    label: string;
    options: Record<string, unknown>;
    show = vi.fn().mockResolvedValue(undefined);
    unminimize = vi.fn().mockResolvedValue(undefined);
    setFocus = vi.fn().mockResolvedValue(undefined);
    emit = vi.fn().mockResolvedValue(undefined);
    once = vi.fn((event: string, callback: (payload?: unknown) => void) => {
      if (event === 'tauri://created') {
        callback();
      }

      return Promise.resolve(() => {});
    });

    constructor(label: string, options: Record<string, unknown>) {
      this.label = label;
      this.options = options;
      createdWindowRecords.push({ label, options });
    }
  }

  return {
    getByLabelMock,
    createdWindowRecords,
    MockWebviewWindow,
  };
});

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/webviewWindow', () => ({
  WebviewWindow: MockWebviewWindow,
  getCurrentWebviewWindow: vi.fn(() => ({ label: 'main' })),
}));

describe('openAiChatWindow', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    createdWindowRecords.length = 0;
    getByLabelMock.mockReset();
    window.history.replaceState({}, '', '/');
    vi.spyOn(crypto, 'randomUUID').mockReturnValue('00000000-0000-4000-8000-000000000123');
  });

  it('会复用现有聊天窗口并发送新的初始化 token', async () => {
    const existingWindow = {
      show: vi.fn().mockResolvedValue(undefined),
      unminimize: vi.fn().mockResolvedValue(undefined),
      setFocus: vi.fn().mockResolvedValue(undefined),
      emit: vi.fn().mockResolvedValue(undefined),
    };

    getByLabelMock.mockResolvedValue(existingWindow);
    mockedInvoke.mockResolvedValue(undefined);

    await openAiChatWindow(
      {
        sourceKey: 'entry-1',
        title: 'Example',
        sourceText: 'hello world',
      },
      {
        windowTitle: 'AI 对话',
      }
    );

    expect(mockedInvoke).toHaveBeenCalledWith('store_ai_chat_window_payload', {
      token: '00000000-0000-4000-8000-000000000123',
      payload: {
        sourceKey: 'entry-1',
        title: 'Example',
        sourceText: 'hello world',
      },
    });
    expect(mockedInvoke).toHaveBeenCalledWith('move_window_to_invoker_monitor', {
      label: AI_CHAT_WINDOW_LABEL,
    });
    expect(existingWindow.emit).toHaveBeenCalledWith(AI_CHAT_WINDOW_NAVIGATE_EVENT, {
      token: '00000000-0000-4000-8000-000000000123',
    });
    expect(createdWindowRecords).toHaveLength(0);
  });

  it('会创建新的聊天窗口并带上 view 与 token 参数', async () => {
    getByLabelMock.mockResolvedValue(null);
    mockedInvoke.mockImplementation(async (command) => {
      if (command === 'get_invoker_monitor_centered_position') {
        return {
          x: 120,
          y: 80,
        };
      }

      return undefined;
    });

    await openAiChatWindow(
      {
        sourceKey: 'entry-2',
        title: 'Second Entry',
        sourceText: 'extract urls',
      },
      {
        windowTitle: 'AI 对话',
      }
    );

    expect(createdWindowRecords).toHaveLength(1);
    expect(createdWindowRecords[0].label).toBe(AI_CHAT_WINDOW_LABEL);
    expect(createdWindowRecords[0].options.title).toBe('AI 对话');
    expect(String(createdWindowRecords[0].options.url)).toContain('view=ai-chat');
    expect(String(createdWindowRecords[0].options.url)).toContain(
      'token=00000000-0000-4000-8000-000000000123'
    );
    expect(createdWindowRecords[0].options.x).toBe(120);
    expect(createdWindowRecords[0].options.y).toBe(80);
    expect(mockedInvoke).toHaveBeenCalledWith('get_invoker_monitor_centered_position', {
      width: 760,
      height: 680,
    });
    expect(mockedInvoke).toHaveBeenCalledWith('move_window_to_invoker_monitor', {
      label: AI_CHAT_WINDOW_LABEL,
    });
  });
});
