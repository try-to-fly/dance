import { invoke } from '@tauri-apps/api/core';
import { create } from 'zustand';
import {
  AiChatMessage,
  AiDialogMode,
  AiSessionState,
  AiTranslationState,
  ProcessTextResponse,
} from '../types/ai';

const TRANSLATE_TO_CHINESE_PROMPT = [
  '请将这段原始文本准确翻译为简体中文，并严格对应原文内容。',
  '只输出译文，不要添加说明、总结、前言或注释。',
  '不要遗漏、合并、扩写或猜测原文没有的信息；遇到歧义时，采用更保守、贴近原文的译法。',
  '保留原文的段落结构、标题、列表层级、编号、表格、代码块、内联代码、链接、路径、命令、占位符、变量名、API/库名、错误信息格式、数字与单位。',
  '有固定中文译法的通用术语请使用准确中文；专有名词或不宜翻译的技术标识保留原文。',
  '如果原文已经是简体中文，仅做必要的轻微校正，不要改写原意。',
].join('');

const createTranslationState = (): AiTranslationState => ({
  status: 'idle',
  content: '',
  error: null,
  model: null,
});

const createMessage = (
  role: AiChatMessage['role'],
  content: string,
  extra?: Partial<AiChatMessage>
): AiChatMessage => ({
  id:
    typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function'
      ? crypto.randomUUID()
      : `${role}-${Date.now()}-${Math.random().toString(16).slice(2)}`,
  role,
  content,
  createdAt: Date.now(),
  ...extra,
});

const createSession = (sourceKey: string, title: string, sourceText: string): AiSessionState => ({
  sourceKey,
  title,
  sourceText,
  input: '',
  loading: false,
  messages: [],
  translation: createTranslationState(),
});

interface OpenDialogPayload {
  sourceKey: string;
  title: string;
  sourceText: string;
  mode: AiDialogMode;
}

interface AiStore {
  isOpen: boolean;
  mode: AiDialogMode;
  activeSourceKey: string | null;
  sessions: Record<string, AiSessionState>;
  openDialog: (payload: OpenDialogPayload) => Promise<void>;
  closeDialog: () => void;
  setMode: (mode: AiDialogMode) => void;
  setInput: (value: string) => void;
  requestTranslation: () => Promise<void>;
  sendPrompt: (prompt?: string) => Promise<void>;
  clearConversation: () => void;
}

const getActiveSession = (state: Pick<AiStore, 'activeSourceKey' | 'sessions'>) =>
  state.activeSourceKey ? state.sessions[state.activeSourceKey] : undefined;

export const useAiStore = create<AiStore>((set, get) => ({
  isOpen: false,
  mode: 'chat',
  activeSourceKey: null,
  sessions: {},

  openDialog: async ({ sourceKey, title, sourceText, mode }) => {
    set((state) => ({
      isOpen: true,
      mode,
      activeSourceKey: sourceKey,
      sessions: {
        ...state.sessions,
        [sourceKey]: state.sessions[sourceKey]
          ? {
              ...state.sessions[sourceKey],
              title,
              sourceText,
            }
          : createSession(sourceKey, title, sourceText),
      },
    }));
  },

  closeDialog: () => {
    set({ isOpen: false });
  },

  setMode: (mode) => {
    set({ mode });
  },

  setInput: (value) => {
    set((state) => {
      const session = getActiveSession(state);
      if (!session || !state.activeSourceKey) {
        return state;
      }

      return {
        sessions: {
          ...state.sessions,
          [state.activeSourceKey]: {
            ...session,
            input: value,
          },
        },
      };
    });
  },

  requestTranslation: async () => {
    const state = get();
    const session = getActiveSession(state);
    if (!session || !state.activeSourceKey) {
      return;
    }

    if (session.loading || session.translation.status === 'loading') {
      return;
    }

    set((current) => {
      const currentSession = getActiveSession(current);
      if (!currentSession || !current.activeSourceKey) {
        return current;
      }

      return {
        sessions: {
          ...current.sessions,
          [current.activeSourceKey]: {
            ...currentSession,
            translation: {
              ...currentSession.translation,
              status: 'loading',
              error: null,
            },
          },
        },
      };
    });

    try {
      const response = await invoke<ProcessTextResponse>('process_text_with_llm', {
        request: {
          source_text: session.sourceText,
          conversation: [],
          user_prompt: TRANSLATE_TO_CHINESE_PROMPT,
        },
      });

      set((current) => {
        const currentSession = getActiveSession(current);
        if (!currentSession || !current.activeSourceKey) {
          return current;
        }

        return {
          sessions: {
            ...current.sessions,
            [current.activeSourceKey]: {
              ...currentSession,
              translation: {
                status: 'success',
                content: response.content,
                error: null,
                model: response.model,
              },
            },
          },
        };
      });
    } catch (error) {
      set((current) => {
        const currentSession = getActiveSession(current);
        if (!currentSession || !current.activeSourceKey) {
          return current;
        }

        return {
          sessions: {
            ...current.sessions,
            [current.activeSourceKey]: {
              ...currentSession,
              translation: {
                ...currentSession.translation,
                status: 'error',
                error: String(error),
              },
            },
          },
        };
      });
    }
  },

  sendPrompt: async (prompt) => {
    const state = get();
    const session = getActiveSession(state);
    if (!session || !state.activeSourceKey || session.loading) {
      return;
    }

    const nextPrompt = (prompt ?? session.input).trim();
    if (!nextPrompt) {
      return;
    }

    const userMessage = createMessage('user', nextPrompt);
    const conversation = session.messages
      .filter((message) => !message.error)
      .map((message) => ({
        role: message.role,
        content: message.content,
      }));

    set((current) => {
      const currentSession = getActiveSession(current);
      if (!currentSession || !current.activeSourceKey) {
        return current;
      }

      return {
        sessions: {
          ...current.sessions,
          [current.activeSourceKey]: {
            ...currentSession,
            loading: true,
            input: '',
            messages: [...currentSession.messages, userMessage],
          },
        },
      };
    });

    try {
      const response = await invoke<ProcessTextResponse>('process_text_with_llm', {
        request: {
          source_text: session.sourceText,
          conversation,
          user_prompt: nextPrompt,
        },
      });

      set((current) => {
        const currentSession = getActiveSession(current);
        if (!currentSession || !current.activeSourceKey) {
          return current;
        }

        return {
          sessions: {
            ...current.sessions,
            [current.activeSourceKey]: {
              ...currentSession,
              loading: false,
              messages: [
                ...currentSession.messages,
                createMessage('assistant', response.content, {
                  model: response.model,
                }),
              ],
            },
          },
        };
      });
    } catch (error) {
      set((current) => {
        const currentSession = getActiveSession(current);
        if (!currentSession || !current.activeSourceKey) {
          return current;
        }

        return {
          sessions: {
            ...current.sessions,
            [current.activeSourceKey]: {
              ...currentSession,
              loading: false,
              messages: [
                ...currentSession.messages,
                createMessage('assistant', String(error), {
                  error: true,
                }),
              ],
            },
          },
        };
      });
    }
  },

  clearConversation: () => {
    set((state) => {
      const session = getActiveSession(state);
      if (!session || !state.activeSourceKey) {
        return state;
      }

      return {
        sessions: {
          ...state.sessions,
          [state.activeSourceKey]: {
            ...session,
            loading: false,
            input: '',
            messages: [],
          },
        },
      };
    });
  },
}));
