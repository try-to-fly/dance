export type AiDialogMode = 'translate' | 'chat';

export type AiMessageRole = 'user' | 'assistant';

export interface AiConversationMessage {
  role: AiMessageRole;
  content: string;
}

export interface ProcessTextRequest {
  source_text: string;
  source_image_data_url?: string | null;
  conversation: AiConversationMessage[];
  user_prompt: string;
}

export interface ProcessTextResponse {
  content: string;
  model: string;
}

export interface AiSessionPayload {
  sourceKey: string;
  title: string;
  sourceText: string;
  sourceImageDataUrl?: string | null;
  mode: AiDialogMode;
}

export interface AiChatWindowPayload {
  sourceKey: string;
  title: string;
  sourceText: string;
  sourceImageDataUrl?: string | null;
}

export interface AiChatMessage extends AiConversationMessage {
  id: string;
  createdAt: number;
  model?: string;
  error?: boolean;
}

export interface AiTranslationState {
  status: 'idle' | 'loading' | 'success' | 'error';
  content: string;
  error: string | null;
  model: string | null;
}

export interface AiSessionState {
  sourceKey: string;
  title: string;
  sourceText: string;
  sourceImageDataUrl: string | null;
  input: string;
  loading: boolean;
  messages: AiChatMessage[];
  translation: AiTranslationState;
}
