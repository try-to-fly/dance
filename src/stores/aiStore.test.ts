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
          '请将这段原始文本准确翻译为简体中文，并严格对应原文内容。只输出译文，不要添加说明、总结、前言或注释。不要遗漏、合并、扩写或猜测原文没有的信息；遇到歧义时，采用更保守、贴近原文的译法。保留原文的段落结构、标题、列表层级、编号、表格、代码块、内联代码、链接、路径、命令、占位符、变量名、API/库名、错误信息格式、数字与单位。有固定中文译法的通用术语请使用准确中文；专有名词或不宜翻译的技术标识保留原文。如果原文已经是简体中文，仅做必要的轻微校正，不要改写原意。',
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

  it('sendPrompt 在没有初始文本时也会发起普通对话请求', async () => {
    mockedInvoke.mockResolvedValue({
      content: '当然，可以先告诉我你的目标。',
      model: 'gpt-4.1-mini',
    });

    await useAiStore.getState().openDialog({
      sourceKey: 'global-chat',
      title: '',
      sourceText: '',
      mode: 'chat',
    });

    useAiStore.getState().setInput('帮我写一个 URL 正则');
    await useAiStore.getState().sendPrompt();

    expect(mockedInvoke).toHaveBeenCalledWith('process_text_with_llm', {
      request: {
        source_text: '',
        conversation: [],
        user_prompt: '帮我写一个 URL 正则',
      },
    });

    const session = useAiStore.getState().sessions['global-chat'];
    expect(session.messages).toHaveLength(2);
    expect(session.messages[0]).toMatchObject({
      role: 'user',
      content: '帮我写一个 URL 正则',
    });
    expect(session.messages[1]).toMatchObject({
      role: 'assistant',
      content: '当然，可以先告诉我你的目标。',
      model: 'gpt-4.1-mini',
    });
  });
});
