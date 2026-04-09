import { invoke } from '@tauri-apps/api/core';
import { WebviewWindow, getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { formatUnknownError } from '../errors';
import type { AiChatWindowPayload } from '../../types/ai';

export const AI_CHAT_WINDOW_LABEL = 'ai-chat';
export const AI_CHAT_WINDOW_VIEW = 'ai-chat';
export const AI_CHAT_WINDOW_NAVIGATE_EVENT = 'ai-chat-window:navigate';

export interface AiChatWindowNavigationPayload {
  token: string;
}

interface OpenAiChatWindowOptions {
  windowTitle?: string;
}

interface WindowPosition {
  x: number;
  y: number;
}

const AI_CHAT_WINDOW_WIDTH = 760;
const AI_CHAT_WINDOW_HEIGHT = 680;
const AI_CHAT_WINDOW_MIN_WIDTH = 520;
const AI_CHAT_WINDOW_MIN_HEIGHT = 480;

const createPayloadToken = () =>
  typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function'
    ? crypto.randomUUID()
    : `ai-chat-${Date.now()}-${Math.random().toString(16).slice(2)}`;

const buildAiChatWindowUrl = (token: string) => {
  const url = new URL(window.location.href);
  url.searchParams.set('view', AI_CHAT_WINDOW_VIEW);
  url.searchParams.set('token', token);
  return url.toString();
};

const moveWindowToInvokerMonitor = (label: string) =>
  invoke('move_window_to_invoker_monitor', { label });

const getInvokerMonitorCenteredPosition = () =>
  invoke<WindowPosition>('get_invoker_monitor_centered_position', {
    width: AI_CHAT_WINDOW_WIDTH,
    height: AI_CHAT_WINDOW_HEIGHT,
  });

export const isAiChatWindowView = () => {
  if (typeof window === 'undefined') {
    return false;
  }

  const params = new URLSearchParams(window.location.search);
  if (params.get('view') === AI_CHAT_WINDOW_VIEW) {
    return true;
  }

  try {
    return getCurrentWebviewWindow().label === AI_CHAT_WINDOW_LABEL;
  } catch {
    return false;
  }
};

export const getAiChatWindowTokenFromLocation = () => {
  if (typeof window === 'undefined') {
    return null;
  }

  return new URLSearchParams(window.location.search).get('token');
};

export async function openAiChatWindow(
  payload: AiChatWindowPayload,
  options: OpenAiChatWindowOptions = {}
) {
  try {
    const token = createPayloadToken();

    await invoke('store_ai_chat_window_payload', {
      token,
      payload,
    });

    const existingWindow = await WebviewWindow.getByLabel(AI_CHAT_WINDOW_LABEL);
    if (existingWindow) {
      await moveWindowToInvokerMonitor(AI_CHAT_WINDOW_LABEL);
      await existingWindow.show();
      await existingWindow.unminimize();
      await existingWindow.setFocus();
      await existingWindow.emit(AI_CHAT_WINDOW_NAVIGATE_EVENT, {
        token,
      } satisfies AiChatWindowNavigationPayload);
      return existingWindow;
    }

    const initialPosition = await getInvokerMonitorCenteredPosition().catch(() => null);

    const chatWindow = new WebviewWindow(AI_CHAT_WINDOW_LABEL, {
      url: buildAiChatWindowUrl(token),
      title: options.windowTitle ?? 'AI Chat',
      width: AI_CHAT_WINDOW_WIDTH,
      height: AI_CHAT_WINDOW_HEIGHT,
      minWidth: AI_CHAT_WINDOW_MIN_WIDTH,
      minHeight: AI_CHAT_WINDOW_MIN_HEIGHT,
      ...(initialPosition
        ? {
            x: initialPosition.x,
            y: initialPosition.y,
          }
        : {
            center: true,
          }),
      focus: true,
      resizable: true,
      decorations: true,
      titleBarStyle: 'overlay',
      hiddenTitle: true,
    });

    return await new Promise<WebviewWindow>((resolve, reject) => {
      const resolveCreated = async () => {
        try {
          await moveWindowToInvokerMonitor(AI_CHAT_WINDOW_LABEL);
          await chatWindow.show();
          await chatWindow.unminimize();
          await chatWindow.setFocus();
          resolve(chatWindow);
        } catch (error) {
          reject(new Error(formatUnknownError(error)));
        }
      };

      chatWindow.once('tauri://created', () => {
        void resolveCreated();
      });
      chatWindow.once('tauri://error', (error) => {
        reject(new Error(formatUnknownError(error)));
      });
    });
  } catch (error) {
    throw new Error(formatUnknownError(error));
  }
}
