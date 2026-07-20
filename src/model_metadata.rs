use codex_core::config::Config;
use codex_models_manager::bundled_models_response;
use codex_models_manager::model_info::model_info_from_slug;
use codex_protocol::openai_models::InputModality;
use codex_protocol::openai_models::ModelInfo;
use codex_protocol::openai_models::ModelVisibility;
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::openai_models::ReasoningEffortPreset;
use serde::Deserialize;
use std::io;

const MODEL_METADATA_ENV: &str = "JAZ_CODEX_MODEL_METADATA";

#[derive(Debug, Deserialize)]
struct ModelMetadata {
    id: String,
    display_name: String,
    #[serde(default)]
    description: String,
    context_window: i64,
    #[serde(default)]
    input_modalities: Vec<String>,
    #[serde(default)]
    reasoning_efforts: Vec<String>,
    #[serde(default)]
    default_reasoning_effort: String,
}

pub(crate) fn apply_env_model_metadata(config: &mut Config) -> io::Result<()> {
    let Ok(raw) = std::env::var(MODEL_METADATA_ENV) else {
        return Ok(());
    };
    let metadata = serde_json::from_str(&raw).map_err(invalid_metadata)?;
    let model = model_info(metadata).map_err(invalid_metadata)?;
    let mut catalog = match config.model_catalog.take() {
        Some(catalog) => catalog,
        None => bundled_models_response().map_err(invalid_metadata)?,
    };
    catalog
        .models
        .retain(|candidate| candidate.slug != model.slug);
    catalog.models.push(model);
    config.model_catalog = Some(catalog);
    Ok(())
}

fn model_info(metadata: ModelMetadata) -> Result<ModelInfo, String> {
    let id = metadata.id.trim();
    if id.is_empty() || metadata.context_window <= 0 {
        return Err("id and a positive context_window are required".to_string());
    }
    let mut model = model_info_from_slug(id);
    model.display_name = metadata.display_name.trim().to_string();
    if model.display_name.is_empty() {
        model.display_name = id.to_string();
    }
    model.description =
        (!metadata.description.trim().is_empty()).then(|| metadata.description.trim().to_string());
    model.context_window = Some(metadata.context_window);
    model.max_context_window = Some(metadata.context_window);
    model.visibility = ModelVisibility::List;
    model.supported_reasoning_levels = metadata
        .reasoning_efforts
        .into_iter()
        .map(|effort| {
            let effort: ReasoningEffort = effort
                .trim()
                .parse()
                .map_err(|err| format!("invalid reasoning effort: {err}"))?;
            Ok(ReasoningEffortPreset {
                description: format!("{} reasoning effort", effort.as_str()),
                effort,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    model.default_reasoning_level = match metadata.default_reasoning_effort.trim() {
        "" => None,
        effort => Some(
            effort
                .parse()
                .map_err(|err| format!("invalid default reasoning effort: {err}"))?,
        ),
    };
    if !metadata.input_modalities.is_empty() {
        model.input_modalities = metadata
            .input_modalities
            .into_iter()
            .map(|modality| match modality.trim() {
                "text" => Ok(InputModality::Text),
                "image" => Ok(InputModality::Image),
                modality => Err(format!("unsupported input modality {modality:?}")),
            })
            .collect::<Result<Vec<_>, String>>()?;
    }
    model.used_fallback_model_metadata = false;
    Ok(model)
}

fn invalid_metadata(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("invalid {MODEL_METADATA_ENV}: {error}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_known_model_from_provider_metadata() {
        let metadata = serde_json::from_str(
            r#"{
                "id":"moonshotai/kimi-k3",
                "display_name":"Kimi K3",
                "description":"Agentic reasoning model",
                "context_window":1048576,
                "input_modalities":["text","image"],
                "reasoning_efforts":["low","high","max"],
                "default_reasoning_effort":"max"
            }"#,
        )
        .expect("metadata should parse");

        let model = model_info(metadata).expect("metadata should produce a model");

        assert_eq!(model.slug, "moonshotai/kimi-k3");
        assert_eq!(model.context_window, Some(1_048_576));
        assert_eq!(model.max_context_window, Some(1_048_576));
        assert_eq!(
            model.input_modalities,
            vec![InputModality::Text, InputModality::Image]
        );
        assert_eq!(
            model
                .supported_reasoning_levels
                .iter()
                .map(|preset| preset.effort.as_str())
                .collect::<Vec<_>>(),
            vec!["low", "high", "max"]
        );
        assert_eq!(
            model
                .default_reasoning_level
                .as_ref()
                .map(|effort| effort.as_str()),
            Some("max")
        );
        assert!(!model.used_fallback_model_metadata);
    }
}
