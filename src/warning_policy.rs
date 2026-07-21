const HIDDEN_PREFIXES: [&str; 2] = [
    "Falling back from WebSockets to HTTPS transport.",
    "Model metadata for `",
];

pub(crate) fn user_visible(message: &str) -> bool {
    !HIDDEN_PREFIXES
        .iter()
        .any(|prefix| message.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hides_internal_diagnostics_only() {
        assert!(!user_visible(
            "Falling back from WebSockets to HTTPS transport. disconnected"
        ));
        assert!(!user_visible(
            "Model metadata for `qwen3.8-max-preview` not found."
        ));
        assert!(user_visible("Approval interrupted"));
    }
}
