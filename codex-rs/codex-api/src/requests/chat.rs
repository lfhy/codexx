use crate::common::ResponsesApiRequest;
use crate::requests::headers::build_session_headers;
use crate::requests::headers::insert_header;
use crate::requests::headers::subagent_header;
use codex_protocol::models::ContentItem;
use codex_protocol::models::FunctionCallOutputBody;
use codex_protocol::models::FunctionCallOutputContentItem;
use codex_protocol::models::ReasoningItemContent;
use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::SessionSource;
use http::HeaderMap;
use serde_json::Value;
use serde_json::json;
use std::collections::HashMap;

pub(crate) struct ChatRequest {
    pub body: Value,
    pub headers: HeaderMap,
}

pub(crate) fn build_chat_request(
    request: &ResponsesApiRequest,
    session_id: Option<String>,
    thread_id: Option<String>,
    session_source: Option<SessionSource>,
    extra_headers: HeaderMap,
) -> ChatRequest {
    let mut messages = Vec::<Value>::new();
    if !request.instructions.is_empty() {
        messages.push(json!({
            "role": "system",
            "content": request.instructions,
        }));
    }

    let mut reasoning_by_anchor_index: HashMap<usize, String> = HashMap::new();
    let mut last_emitted_role: Option<&str> = None;
    for item in &request.input {
        match item {
            ResponseItem::Message { role, .. } => last_emitted_role = Some(role.as_str()),
            ResponseItem::FunctionCall { .. }
            | ResponseItem::LocalShellCall { .. }
            | ResponseItem::CustomToolCall { .. }
            | ResponseItem::ToolSearchCall { .. } => last_emitted_role = Some("assistant"),
            ResponseItem::FunctionCallOutput { .. }
            | ResponseItem::CustomToolCallOutput { .. }
            | ResponseItem::ToolSearchOutput { .. } => last_emitted_role = Some("tool"),
            ResponseItem::Reasoning { .. }
            | ResponseItem::WebSearchCall { .. }
            | ResponseItem::ImageGenerationCall { .. }
            | ResponseItem::Compaction { .. }
            | ResponseItem::ContextCompaction { .. }
            | ResponseItem::Other => {}
        }
    }

    let mut last_user_index: Option<usize> = None;
    for (idx, item) in request.input.iter().enumerate() {
        if let ResponseItem::Message { role, .. } = item
            && role == "user"
        {
            last_user_index = Some(idx);
        }
    }

    if !matches!(last_emitted_role, Some("user")) {
        for (idx, item) in request.input.iter().enumerate() {
            if let Some(user_idx) = last_user_index
                && idx <= user_idx
            {
                continue;
            }

            let ResponseItem::Reasoning {
                content: Some(items),
                ..
            } = item
            else {
                continue;
            };

            let mut text = String::new();
            for entry in items {
                match entry {
                    ReasoningItemContent::ReasoningText { text: segment }
                    | ReasoningItemContent::Text { text: segment } => text.push_str(segment),
                }
            }
            if text.trim().is_empty() {
                continue;
            }

            let mut attached = false;
            if idx > 0
                && let ResponseItem::Message { role, .. } = &request.input[idx - 1]
                && role == "assistant"
            {
                reasoning_by_anchor_index
                    .entry(idx - 1)
                    .and_modify(|value| value.push_str(&text))
                    .or_insert(text.clone());
                attached = true;
            }

            if !attached && idx + 1 < request.input.len() {
                match &request.input[idx + 1] {
                    ResponseItem::FunctionCall { .. }
                    | ResponseItem::LocalShellCall { .. }
                    | ResponseItem::CustomToolCall { .. }
                    | ResponseItem::ToolSearchCall { .. } => {
                        reasoning_by_anchor_index
                            .entry(idx + 1)
                            .and_modify(|value| value.push_str(&text))
                            .or_insert(text.clone());
                    }
                    ResponseItem::Message { role, .. } if role == "assistant" => {
                        reasoning_by_anchor_index
                            .entry(idx + 1)
                            .and_modify(|value| value.push_str(&text))
                            .or_insert(text.clone());
                    }
                    _ => {}
                }
            }
        }
    }

    let mut last_assistant_text: Option<String> = None;
    for (idx, item) in request.input.iter().enumerate() {
        match item {
            ResponseItem::Message { role, content, .. } => {
                let mut text = String::new();
                let mut items = Vec::<Value>::new();
                let mut saw_image = false;

                for content_item in content {
                    match content_item {
                        ContentItem::InputText { text: value }
                        | ContentItem::OutputText { text: value } => {
                            text.push_str(value);
                            items.push(json!({"type":"text","text": value}));
                        }
                        ContentItem::InputImage {
                            image_url,
                            detail: _,
                        } => {
                            saw_image = true;
                            items.push(json!({
                                "type":"image_url",
                                "image_url": { "url": image_url },
                            }));
                        }
                    }
                }

                if role == "assistant" {
                    if let Some(previous) = &last_assistant_text
                        && previous == &text
                    {
                        continue;
                    }
                    last_assistant_text = Some(text.clone());
                }

                let content_value = if role == "assistant" {
                    json!(text)
                } else if saw_image {
                    json!(items)
                } else {
                    json!(text)
                };

                let mut message = json!({
                    "role": role,
                    "content": content_value,
                });
                if role == "assistant"
                    && let Some(reasoning) = reasoning_by_anchor_index.get(&idx)
                    && let Some(object) = message.as_object_mut()
                {
                    object.insert("reasoning".to_string(), json!(reasoning));
                }
                messages.push(message);
            }
            ResponseItem::FunctionCall {
                namespace: _,
                name,
                arguments,
                call_id,
                ..
            } => {
                let reasoning = reasoning_by_anchor_index.get(&idx).map(String::as_str);
                push_tool_call_message(
                    &mut messages,
                    json!({
                        "id": call_id,
                        "type": "function",
                        "function": {
                            "name": name,
                            "arguments": arguments,
                        }
                    }),
                    reasoning,
                );
            }
            ResponseItem::LocalShellCall {
                id,
                call_id,
                status,
                action,
            } => {
                let reasoning = reasoning_by_anchor_index.get(&idx).map(String::as_str);
                push_tool_call_message(
                    &mut messages,
                    json!({
                        "id": call_id.clone().or_else(|| id.clone()).unwrap_or_default(),
                        "type": "local_shell_call",
                        "status": status,
                        "action": action,
                    }),
                    reasoning,
                );
            }
            ResponseItem::FunctionCallOutput { call_id, output } => {
                messages.push(json!({
                    "role": "tool",
                    "tool_call_id": call_id,
                    "content": function_call_output_json(&output.body),
                }));
            }
            ResponseItem::CustomToolCall {
                call_id,
                name,
                input,
                ..
            } => {
                let reasoning = reasoning_by_anchor_index.get(&idx).map(String::as_str);
                push_tool_call_message(
                    &mut messages,
                    json!({
                        "id": call_id,
                        "type": "custom",
                        "custom": {
                            "name": name,
                            "input": input,
                        }
                    }),
                    reasoning,
                );
            }
            ResponseItem::CustomToolCallOutput {
                call_id, output, ..
            } => {
                messages.push(json!({
                    "role": "tool",
                    "tool_call_id": call_id,
                    "content": function_call_output_json(&output.body),
                }));
            }
            ResponseItem::Reasoning { .. }
            | ResponseItem::WebSearchCall { .. }
            | ResponseItem::ImageGenerationCall { .. }
            | ResponseItem::Compaction { .. }
            | ResponseItem::ContextCompaction { .. }
            | ResponseItem::ToolSearchCall { .. }
            | ResponseItem::ToolSearchOutput { .. }
            | ResponseItem::Other => {}
        }
    }

    let mut body = json!({
        "model": request.model,
        "messages": messages,
        "stream": true,
        "tools": request.tools,
    });
    if !request.tool_choice.is_empty()
        && let Some(object) = body.as_object_mut()
    {
        object.insert("tool_choice".to_string(), json!(request.tool_choice));
        object.insert(
            "parallel_tool_calls".to_string(),
            json!(request.parallel_tool_calls),
        );
    }

    let mut headers = build_session_headers(session_id, thread_id.clone());
    if let Some(ref thread_id) = thread_id {
        insert_header(&mut headers, "x-client-request-id", thread_id);
    }
    headers.extend(extra_headers);
    if let Some(subagent) = subagent_header(&session_source) {
        insert_header(&mut headers, "x-openai-subagent", &subagent);
    }

    ChatRequest { body, headers }
}

fn function_call_output_json(output: &FunctionCallOutputBody) -> Value {
    match output {
        FunctionCallOutputBody::Text(content) => json!(content),
        FunctionCallOutputBody::ContentItems(items) => json!(
            items
                .iter()
                .map(|item| match item {
                    FunctionCallOutputContentItem::InputText { text } => {
                        json!({"type":"text","text": text})
                    }
                    FunctionCallOutputContentItem::InputImage {
                        image_url,
                        detail: _,
                    } => {
                        json!({"type":"image_url","image_url": { "url": image_url }})
                    }
                })
                .collect::<Vec<_>>()
        ),
    }
}

fn push_tool_call_message(messages: &mut Vec<Value>, tool_call: Value, reasoning: Option<&str>) {
    if let Some(Value::Object(object)) = messages.last_mut()
        && object.get("role").and_then(Value::as_str) == Some("assistant")
        && object.get("content").is_some_and(Value::is_null)
        && let Some(tool_calls) = object.get_mut("tool_calls").and_then(Value::as_array_mut)
    {
        tool_calls.push(tool_call);
        if let Some(reasoning) = reasoning {
            if let Some(Value::String(existing)) = object.get_mut("reasoning") {
                if !existing.is_empty() {
                    existing.push('\n');
                }
                existing.push_str(reasoning);
            } else {
                object.insert(
                    "reasoning".to_string(),
                    Value::String(reasoning.to_string()),
                );
            }
        }
        return;
    }

    let mut message = json!({
        "role": "assistant",
        "content": null,
        "tool_calls": [tool_call],
    });
    if let Some(reasoning) = reasoning
        && let Some(object) = message.as_object_mut()
    {
        object.insert("reasoning".to_string(), json!(reasoning));
    }
    messages.push(message);
}
