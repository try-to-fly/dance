import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ChatButton, GLOBAL_AI_CHAT_SOURCE_KEY } from './chat-button';

const { mockedOpenAiChatWindow } = vi.hoisted(() => ({
  mockedOpenAiChatWindow: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('../lib/ai/chatWindow', () => ({
  openAiChatWindow: mockedOpenAiChatWindow,
}));

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const dictionary: Record<string, string> = {
        'detail.ai.chatButton': 'Chat',
        'detail.ai.chatTitleStandalone': 'AI 对话',
      };

      return dictionary[key] ?? key;
    },
  }),
}));

describe('ChatButton', () => {
  it('点击后会打开不带初始文本的 chat 窗口', async () => {
    render(<ChatButton />);

    fireEvent.click(screen.getByRole('button', { name: 'Chat' }));

    await waitFor(() => {
      expect(mockedOpenAiChatWindow).toHaveBeenCalledWith(
        {
          sourceKey: GLOBAL_AI_CHAT_SOURCE_KEY,
          title: '',
          sourceText: '',
        },
        {
          windowTitle: 'AI 对话',
        }
      );
    });
  });
});
