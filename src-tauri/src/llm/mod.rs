use crate::config::LlmConfig;
use anyhow::{anyhow, Context, Result};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use url::Url;

const DEFAULT_LLM_BASE_URL: &str = "https://api.openai.com/v1";
const MAX_SOURCE_CHARS: usize = 20_000;
const MAX_PROMPT_CHARS: usize = 4_000;
const MAX_CONTEXT_MESSAGES: usize = 16;
const CONNECTION_TEST_SOURCE_TEXT: &str =
    "Dance preferences connection test. 这是一条来自 Dance 偏好设置的连通性测试文本。";
const CONNECTION_TEST_PROMPT: &str =
    "Reply with a short confirmation to indicate the request succeeded.";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LlmMessageRole {
    User,
    Assistant,
}

impl LlmMessageRole {
    fn as_api_role(&self) -> &'static str {
        match self {
            LlmMessageRole::User => "user",
            LlmMessageRole::Assistant => "assistant",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConversationMessage {
    pub role: LlmMessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessTextRequest {
    pub source_text: String,
    #[serde(default)]
    pub conversation: Vec<LlmConversationMessage>,
    pub user_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessTextResponse {
    pub content: String,
    pub model: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionsResponse {
    model: Option<String>,
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    content: Option<ChatContent>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ChatContent {
    Text(String),
    Parts(Vec<ChatContentPart>),
}

impl ChatContent {
    fn into_text(self) -> String {
        match self {
            ChatContent::Text(value) => value,
            ChatContent::Parts(parts) => parts
                .into_iter()
                .filter_map(|part| match part {
                    ChatContentPart::Text { text, .. } => text,
                    ChatContentPart::Refusal { refusal, .. } => refusal,
                    ChatContentPart::Unknown => None,
                })
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ChatContentPart {
    #[serde(rename = "text")]
    Text { text: Option<String> },
    #[serde(rename = "refusal")]
    Refusal { refusal: Option<String> },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
struct ApiErrorEnvelope {
    error: ApiErrorPayload,
}

#[derive(Debug, Deserialize)]
struct ApiErrorPayload {
    message: Option<String>,
    #[serde(rename = "type")]
    error_type: Option<String>,
    code: Option<serde_json::Value>,
}

fn ensure_text_limit(label: &str, value: &str, limit: usize) -> Result<()> {
    let length = value.chars().count();
    if length > limit {
        return Err(anyhow!(
            "{}过长（{} 字符），当前上限为 {} 字符。",
            label,
            length,
            limit
        ));
    }

    Ok(())
}

fn build_chat_completions_url(base_url: &str) -> Result<String> {
    let trimmed = if base_url.trim().is_empty() {
        DEFAULT_LLM_BASE_URL
    } else {
        base_url.trim()
    };

    let normalized = trimmed.trim_end_matches('/');
    let endpoint = if normalized.ends_with("/chat/completions") {
        normalized.to_string()
    } else if normalized.ends_with("/v1") {
        format!("{normalized}/chat/completions")
    } else {
        format!("{normalized}/v1/chat/completions")
    };

    Url::parse(&endpoint).context("无效的 Base URL")?;
    Ok(endpoint)
}

fn sanitize_conversation(conversation: Vec<LlmConversationMessage>) -> Vec<LlmConversationMessage> {
    let mut cleaned = conversation
        .into_iter()
        .filter_map(|message| {
            let content = message.content.trim().to_string();
            if content.is_empty() {
                return None;
            }

            Some(LlmConversationMessage {
                role: message.role,
                content,
            })
        })
        .collect::<Vec<_>>();

    if cleaned.len() > MAX_CONTEXT_MESSAGES {
        cleaned = cleaned.split_off(cleaned.len() - MAX_CONTEXT_MESSAGES);
    }

    cleaned
}

fn build_system_prompt(has_source_text: bool) -> &'static str {
    if has_source_text {
        concat!(
            "你是桌面剪贴板应用里的文本处理助手。用户会提供一段固定的原始文本和后续提示词。",
            "你的所有回答都必须基于这段原始文本完成，不要臆造原文中不存在的信息。",
            "如果用户要求翻译、提取 URL、总结、改写、结构化整理或继续追问，都应围绕同一段原始文本处理。"
        )
    } else {
        concat!(
            "你是桌面剪贴板应用里的 AI 助手。",
            "当前对话不一定会提供预置原始文本；如果没有看到原始文本，就按普通对话直接回答。",
            "如果后续消息提供了上下文或约束，请严格依据这些内容作答，不要谎称看到了不存在的原文。"
        )
    }
}

fn build_chat_messages(
    source_text: &str,
    conversation: Vec<LlmConversationMessage>,
    user_prompt: &str,
) -> Vec<serde_json::Value> {
    let has_source_text = !source_text.is_empty();
    let mut messages = vec![json!({
        "role": "system",
        "content": build_system_prompt(has_source_text),
    })];

    if has_source_text {
        messages.push(json!({
            "role": "user",
            "content": format!(
                "以下是当前选中的原始文本，请在整个对话中始终以它为依据：\n\n<source_text>\n{}\n</source_text>",
                source_text
            ),
        }));
    }

    for message in conversation {
        messages.push(json!({
            "role": message.role.as_api_role(),
            "content": message.content,
        }));
    }

    messages.push(json!({
        "role": "user",
        "content": user_prompt,
    }));

    messages
}

pub async fn process_text(
    config: &LlmConfig,
    request: ProcessTextRequest,
) -> Result<ProcessTextResponse> {
    let api_key = config.api_key.trim();
    if api_key.is_empty() {
        return Err(anyhow!("请先在偏好设置中填写 API Key。"));
    }

    let model = config.model.trim();
    if model.is_empty() {
        return Err(anyhow!("请先在偏好设置中填写模型名称。"));
    }

    let source_text = request.source_text.trim();
    if !source_text.is_empty() {
        ensure_text_limit("原始文本", source_text, MAX_SOURCE_CHARS)?;
    }

    let user_prompt = request.user_prompt.trim();
    if user_prompt.is_empty() {
        return Err(anyhow!("请输入提示词。"));
    }

    ensure_text_limit("提示词", user_prompt, MAX_PROMPT_CHARS)?;

    let endpoint = build_chat_completions_url(&config.base_url)?;
    let conversation = sanitize_conversation(request.conversation);
    let messages = build_chat_messages(source_text, conversation, user_prompt);

    let payload = json!({
        "model": model,
        "messages": messages,
    });

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(90))
        .build()
        .context("创建 LLM 客户端失败")?;

    let response = client
        .post(&endpoint)
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .json(&payload)
        .send()
        .await
        .context("LLM 请求发送失败")?;

    let status = response.status();
    let body = response.text().await.context("读取 LLM 响应失败")?;

    if !status.is_success() {
        if let Ok(error) = serde_json::from_str::<ApiErrorEnvelope>(&body) {
            let code = error
                .error
                .code
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let error_type = error
                .error
                .error_type
                .unwrap_or_else(|| "unknown_error".to_string());
            let message = error
                .error
                .message
                .unwrap_or_else(|| "请求失败".to_string());
            return Err(anyhow!(
                "LLM 请求失败（HTTP {} / {} / {}）：{}",
                status.as_u16(),
                error_type,
                code,
                message
            ));
        }

        return Err(anyhow!(
            "LLM 请求失败（HTTP {}）：{}",
            status.as_u16(),
            body
        ));
    }

    let parsed: ChatCompletionsResponse =
        serde_json::from_str(&body).context("解析 LLM 响应失败")?;
    let content = parsed
        .choices
        .into_iter()
        .find_map(|choice| choice.message.content.map(ChatContent::into_text))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("LLM 返回了空内容。"))?;

    Ok(ProcessTextResponse {
        content,
        model: parsed.model.unwrap_or_else(|| model.to_string()),
    })
}

pub async fn test_config(config: &LlmConfig) -> Result<ProcessTextResponse> {
    process_text(
        config,
        ProcessTextRequest {
            source_text: CONNECTION_TEST_SOURCE_TEXT.to_string(),
            conversation: vec![],
            user_prompt: CONNECTION_TEST_PROMPT.to_string(),
        },
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::{
        build_chat_completions_url, build_chat_messages, LlmConversationMessage, LlmMessageRole,
    };

    #[test]
    fn build_chat_url_supports_root_base_url() {
        let endpoint = build_chat_completions_url("https://api.openai.com").unwrap();
        assert_eq!(endpoint, "https://api.openai.com/v1/chat/completions");
    }

    #[test]
    fn build_chat_url_supports_v1_base_url() {
        let endpoint = build_chat_completions_url("https://openrouter.ai/api/v1").unwrap();
        assert_eq!(endpoint, "https://openrouter.ai/api/v1/chat/completions");
    }

    #[test]
    fn build_chat_url_keeps_explicit_chat_endpoint() {
        let endpoint =
            build_chat_completions_url("https://example.com/custom/chat/completions").unwrap();
        assert_eq!(endpoint, "https://example.com/custom/chat/completions");
    }

    #[test]
    fn build_chat_messages_includes_source_context_when_source_exists() {
        let messages = build_chat_messages(
            "hello world",
            vec![LlmConversationMessage {
                role: LlmMessageRole::Assistant,
                content: "收到".to_string(),
            }],
            "总结一下",
        );

        assert_eq!(messages.len(), 4);
        assert_eq!(messages[0]["role"], "system");
        assert!(messages[0]["content"]
            .as_str()
            .unwrap()
            .contains("固定的原始文本"));
        assert_eq!(messages[1]["role"], "user");
        assert!(messages[1]["content"]
            .as_str()
            .unwrap()
            .contains("<source_text>"));
        assert_eq!(messages[2]["role"], "assistant");
        assert_eq!(messages[2]["content"], "收到");
        assert_eq!(messages[3]["role"], "user");
        assert_eq!(messages[3]["content"], "总结一下");
    }

    #[test]
    fn build_chat_messages_skips_source_context_when_source_is_empty() {
        let messages = build_chat_messages("", vec![], "帮我写个正则");

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0]["role"], "system");
        assert!(messages[0]["content"]
            .as_str()
            .unwrap()
            .contains("不一定会提供预置原始文本"));
        assert_eq!(messages[1]["role"], "user");
        assert_eq!(messages[1]["content"], "帮我写个正则");
    }
}
