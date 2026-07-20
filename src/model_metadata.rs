use codex_core::config::Config;
use codex_models_manager::bundled_models_response;
use codex_models_manager::model_info::model_info_from_slug;
use codex_protocol::openai_models::InputModality;
use codex_protocol::openai_models::ModelInfo;
use codex_protocol::openai_models::ModelVisibility;
use codex_protocol::openai_models::ModelsResponse;
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
    let raw = std::env::var(MODEL_METADATA_ENV).ok();
    apply_model_metadata(&mut config.model_catalog, raw.as_deref())
}

fn apply_model_metadata(catalog: &mut Option<ModelsResponse>, raw: Option<&str>) -> io::Result<()> {
    let Some(raw) = raw else {
        return Ok(());
    };
    let metadata = serde_json::from_str(raw).map_err(invalid_metadata)?;
    let model = model_info(metadata).map_err(invalid_metadata)?;
    let mut next = match catalog.take() {
        Some(catalog) => catalog,
        None => bundled_models_response().map_err(invalid_metadata)?,
    };
    next.models.retain(|candidate| candidate.slug != model.slug);
    next.models.push(model);
    *catalog = Some(next);
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
    fn applies_metadata_at_the_catalog_boundary() {
        let mut model_catalog = None;
        apply_model_metadata(&mut model_catalog, None).expect("missing metadata should be ignored");
        assert!(model_catalog.is_none());
        assert!(apply_model_metadata(&mut model_catalog, Some("{")).is_err());
        assert!(model_catalog.is_none());

        let raw = r#"{
                "id":"moonshotai/kimi-k3",
                "display_name":"Kimi K3",
                "description":"Agentic reasoning model",
                "context_window":1048576,
                "input_modalities":["text","image"],
                "reasoning_efforts":["low","high","max"],
                "default_reasoning_effort":"max"
            }"#;
        let mut catalog = bundled_models_response().expect("bundled catalog should load");
        let preserved = catalog
            .models
            .iter()
            .find(|model| model.slug != "moonshotai/kimi-k3")
            .expect("bundled catalog should contain another model")
            .clone();
        let expected_base_instructions =
            model_info_from_slug("moonshotai/kimi-k3").base_instructions;
        catalog
            .models
            .push(model_info_from_slug("moonshotai/kimi-k3"));
        let other_models = catalog
            .models
            .iter()
            .filter(|model| model.slug != "moonshotai/kimi-k3")
            .count();
        model_catalog = Some(catalog);

        apply_model_metadata(&mut model_catalog, Some(raw)).expect("metadata should merge");

        let catalog = model_catalog.expect("catalog should remain present");
        assert_eq!(catalog.models.len(), other_models + 1);
        assert!(catalog.models.contains(&preserved));
        let models = catalog
            .models
            .iter()
            .filter(|model| model.slug == "moonshotai/kimi-k3")
            .collect::<Vec<_>>();
        assert_eq!(models.len(), 1);
        let model = models[0];

        assert_eq!(model.context_window, Some(1_048_576));
        assert_eq!(model.max_context_window, Some(1_048_576));
        assert_eq!(model.base_instructions, expected_base_instructions);
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
