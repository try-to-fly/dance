import { MessageSquareText } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { openAiChatWindow } from '../lib/ai/chatWindow';
import { formatUnknownError } from '../lib/errors';
import { Button, type ButtonProps } from './ui/button';

export const GLOBAL_AI_CHAT_SOURCE_KEY = '__global-ai-chat__';

interface ChatButtonProps {
  buttonClassName?: string;
  variant?: ButtonProps['variant'];
  size?: ButtonProps['size'];
}

export function ChatButton({ buttonClassName, variant = 'outline', size = 'sm' }: ChatButtonProps) {
  const { t } = useTranslation(['common']);

  const handleOpenChat = async () => {
    try {
      await openAiChatWindow(
        {
          sourceKey: GLOBAL_AI_CHAT_SOURCE_KEY,
          title: '',
          sourceText: '',
        },
        {
          windowTitle: t('detail.ai.chatTitleStandalone'),
        }
      );
    } catch (error) {
      console.error('[ChatButton] 打开聊天窗口失败:', formatUnknownError(error));
    }
  };

  return (
    <Button
      variant={variant}
      size={size}
      onClick={() => void handleOpenChat()}
      aria-label={t('detail.ai.chatButton')}
      title={t('detail.ai.chatButton')}
      className={buttonClassName}
    >
      <MessageSquareText className="h-4 w-4" />
      <span>{t('detail.ai.chatButton')}</span>
    </Button>
  );
}
