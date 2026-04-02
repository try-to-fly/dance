import { beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { useAiStore } from './aiStore';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

describe('aiStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useAiStore.setState({
      isOpen: false,
      mode: 'chat',
      activeSourceKey: null,
      sessions: {},
    });
  });

  it('requestTranslation 会基于当前原文发起翻译请求', async () => {
    mockedInvoke.mockResolvedValue({
      content: '这是翻译结果',
      model: 'gpt-4.1-mini',
    });

    await useAiStore.getState().openDialog({
      sourceKey: 'entry-1',
      title: 'hello',
      sourceText: 'hello world',
      mode: 'translate',
    });

    await useAiStore.getState().requestTranslation();

    expect(mockedInvoke).toHaveBeenCalledWith('process_text_with_llm', {
      request: {
        source_text: 'hello world',
        conversation: [],
        user_prompt:
          '请将这段原始文本翻译成自然、准确的简体中文。保留原文中的结构、列表、代码块、链接和关键术语。',
      },
    });

    expect(useAiStore.getState().sessions['entry-1'].translation).toEqual({
      status: 'success',
      content: '这是翻译结果',
      error: null,
      model: 'gpt-4.1-mini',
    });
  });

  it('sendPrompt 会带上当前原文和历史消息', async () => {
    mockedInvoke.mockResolvedValue({
      content: '找到了 1 个 URL',
      model: 'gpt-4.1-mini',
    });

    await useAiStore.getState().openDialog({
      sourceKey: 'entry-1',
      title: 'https://example.com',
      sourceText: 'Visit https://example.com now.',
      mode: 'chat',
    });

    useAiStore.getState().setInput('提取所有 URL');
    await useAiStore.getState().sendPrompt();

    expect(mockedInvoke).toHaveBeenCalledWith('process_text_with_llm', {
      request: {
        source_text: 'Visit https://example.com now.',
        conversation: [],
        user_prompt: '提取所有 URL',
      },
    });

    const session = useAiStore.getState().sessions['entry-1'];
    expect(session.messages).toHaveLength(2);
    expect(session.messages[0]).toMatchObject({
      role: 'user',
      content: '提取所有 URL',
    });
    expect(session.messages[1]).toMatchObject({
      role: 'assistant',
      content: '找到了 1 个 URL',
      model: 'gpt-4.1-mini',
    });
    expect(session.loading).toBe(false);
    expect(session.input).toBe('');
  });
});
