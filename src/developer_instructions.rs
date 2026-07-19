use agent_client_protocol::schema::v1::Meta;
use codex_core::config::Config;
use serde_json::Value;

const CODEX_META_KEY: &str = "codex";
const SYSTEM_PROMPT_META_KEY: &str = "systemPrompt";
const APPEND_DEVELOPER_INSTRUCTIONS_META_KEY: &str = "appendDeveloperInstructions";
const DEVELOPER_INSTRUCTIONS_META_KEY: &str = "developerInstructions";
const APPEND_META_KEY: &str = "append";

pub(crate) fn apply_session_meta_developer_instructions(config: &mut Config, meta: Option<&Meta>) {
    let Some(instructions) = developer_instructions_from_session_meta(meta) else {
        return;
    };

    config.developer_instructions =
        append_developer_instructions(config.developer_instructions.take(), &instructions);
}

pub(crate) fn append_developer_instructions(
    existing: Option<String>,
    additional: &str,
) -> Option<String> {
    let additional = additional.trim();
    if additional.is_empty() {
        return existing;
    }

    match existing {
        Some(existing) if !existing.trim().is_empty() => {
            Some(format!("{}\n\n{}", existing.trim_end(), additional))
        }
        _ => Some(additional.to_string()),
    }
}

fn developer_instructions_from_session_meta(meta: Option<&Meta>) -> Option<String> {
    let meta = meta?;
    let mut parts = Vec::new();

    collect_system_prompt_append(meta.get(SYSTEM_PROMPT_META_KEY), &mut parts);
    collect_text_value(meta.get(APPEND_DEVELOPER_INSTRUCTIONS_META_KEY), &mut parts);

    if let Some(codex_meta) = meta.get(CODEX_META_KEY).and_then(Value::as_object) {
        collect_text_value(
            codex_meta.get(APPEND_DEVELOPER_INSTRUCTIONS_META_KEY),
            &mut parts,
        );
        collect_text_value(codex_meta.get(DEVELOPER_INSTRUCTIONS_META_KEY), &mut parts);
    }

    (!parts.is_empty()).then(|| parts.join("\n\n"))
}

fn collect_system_prompt_append(value: Option<&Value>, parts: &mut Vec<String>) {
    match value {
        Some(Value::String(_)) => collect_text_value(value, parts),
        Some(Value::Object(object)) => {
            collect_text_value(object.get(APPEND_META_KEY), parts);
            collect_text_value(object.get(APPEND_DEVELOPER_INSTRUCTIONS_META_KEY), parts);
        }
        _ => {}
    }
}

fn collect_text_value(value: Option<&Value>, parts: &mut Vec<String>) {
    match value {
        Some(Value::String(text)) => push_trimmed(text, parts),
        Some(Value::Array(values)) => {
            for value in values {
                if let Value::String(text) = value {
                    push_trimmed(text, parts);
                }
            }
        }
        _ => {}
    }
}

fn push_trimmed(text: &str, parts: &mut Vec<String>) {
    let text = text.trim();
    if !text.is_empty() {
        parts.push(text.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_to_existing_developer_instructions() {
        assert_eq!(
            append_developer_instructions(Some("base".to_string()), "extra"),
            Some("base\n\nextra".to_string())
        );
    }

    #[test]
    fn ignores_empty_additional_developer_instructions() {
        assert_eq!(
            append_developer_instructions(Some("base".to_string()), "  "),
            Some("base".to_string())
        );
    }

    #[test]
    fn extracts_codex_namespaced_append_developer_instructions() {
        let meta = Meta::from_iter([(
            CODEX_META_KEY.to_string(),
            serde_json::json!({
                APPEND_DEVELOPER_INSTRUCTIONS_META_KEY: "memory context",
            }),
        )]);

        assert_eq!(
            developer_instructions_from_session_meta(Some(&meta)),
            Some("memory context".to_string())
        );
    }

    #[test]
    fn extracts_system_prompt_append_object() {
        let meta = Meta::from_iter([(
            SYSTEM_PROMPT_META_KEY.to_string(),
            serde_json::json!({
                APPEND_META_KEY: "claude-style append",
            }),
        )]);

        assert_eq!(
            developer_instructions_from_session_meta(Some(&meta)),
            Some("claude-style append".to_string())
        );
    }

    #[test]
    fn treats_system_prompt_string_as_append_for_codex() {
        let meta = Meta::from_iter([(
            SYSTEM_PROMPT_META_KEY.to_string(),
            serde_json::json!("compat append"),
        )]);

        assert_eq!(
            developer_instructions_from_session_meta(Some(&meta)),
            Some("compat append".to_string())
        );
    }
}
