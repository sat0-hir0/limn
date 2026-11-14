//! Feature flags — see `docs/development/feature-flags.md` for the
//! policy (three-stage maturity model, naming conventions, promotion
//! rules). This module implements Pattern B: a single aggregated
//! struct read once at startup.

/// All feature flags active in the current build, read from
/// `LIMN_FEAT_*` environment variables at startup.
#[derive(Debug, Clone, Default)]
pub struct FeatureFlags {
    /// Gates `EditorView` (editable). When off, the app shows the
    /// read-only `DocumentView`.
    pub edit: bool,
}

impl FeatureFlags {
    /// Reads all `LIMN_FEAT_*` env vars. Truthy values (case-insensitive)
    /// are `1`, `true`, `on`, `yes`. Anything else, or an absent var, is
    /// OFF.
    #[must_use]
    pub fn from_env() -> Self {
        Self::from_env_vars(|name| std::env::var(name).ok())
    }

    /// Pure core of `from_env`, parameterized over the var lookup so it
    /// can be exercised without touching real process environment state.
    fn from_env_vars(get_var: impl Fn(&str) -> Option<String>) -> Self {
        Self {
            edit: env_truthy(&get_var, "LIMN_FEAT_EDIT"),
        }
    }
}

fn env_truthy(get_var: &impl Fn(&str) -> Option<String>, name: &str) -> bool {
    matches!(
        get_var(name).map(|v| v.to_ascii_lowercase()).as_deref(),
        Some("1" | "true" | "on" | "yes"),
    )
}

#[cfg(test)]
mod tests {
    use super::FeatureFlags;

    #[test]
    fn defaults_to_off_when_env_var_is_absent() {
        let flags = FeatureFlags::from_env_vars(|_| None);
        assert!(!flags.edit);
    }

    #[test]
    fn recognizes_truthy_values_case_insensitively() {
        for value in ["1", "true", "TRUE", "on", "On", "yes", "YES"] {
            let flags = FeatureFlags::from_env_vars(|name| {
                (name == "LIMN_FEAT_EDIT").then(|| value.to_string())
            });
            assert!(flags.edit, "expected {value:?} to be truthy");
        }
    }

    #[test]
    fn treats_falsy_or_unrecognized_values_as_off() {
        for value in ["0", "false", "off", "no", "banana", ""] {
            let flags = FeatureFlags::from_env_vars(|name| {
                (name == "LIMN_FEAT_EDIT").then(|| value.to_string())
            });
            assert!(!flags.edit, "expected {value:?} to be falsy");
        }
    }
}
