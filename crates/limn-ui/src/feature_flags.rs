//! Feature flags (Pattern B: aggregated struct).
//!
//! See `docs/development/feature-flags.md`. Flags are read once at
//! startup via [`FeatureFlags::from_env`] and stored as a gpui global
//! (`cx.set_global`).
//!
//! Today the global is registered as preparation: `main` branches on
//! the flags directly (the read-only vs editable split), and no view in
//! the widget tree reads them yet. A later wave (`LIMN_FEAT_PALETTE`)
//! will read flags from a widget via `cx.global::<FeatureFlags>()`, so
//! the global registration is kept in place for that.
//!
//! Env vars are named `LIMN_FEAT_<NAME>` and evaluated by value, not
//! presence — `LIMN_FEAT_EDIT=0` explicitly turns the flag off.

use gpui::Global;

/// All currently known feature flags, aggregated into one struct.
///
/// Every field defaults to `false` (flag off), matching "absent env var
/// = off".
///
/// Registered as a gpui global at startup in preparation for widgets
/// reading flags via `cx.global::<FeatureFlags>()`. Currently `main`
/// consults the flags directly when choosing the read-only vs editable
/// path; no widget reads the global yet.
#[derive(Debug, Clone, Default)]
pub struct FeatureFlags {
    /// `LIMN_FEAT_EDIT` — editable editor (M2). When off, the read-only
    /// `DocumentView` is shown.
    pub edit: bool,
    /// `LIMN_FEAT_PALETTE` — slash / command palette. Not yet wired.
    pub palette: bool,
    /// `LIMN_FEAT_AI` — AI integration. Not yet wired.
    pub ai: bool,
}

impl FeatureFlags {
    /// Read all `LIMN_FEAT_*` env vars at startup. Truthy values
    /// (case-insensitive) are `1`, `true`, `on`, `yes`. Anything else
    /// (or absent) is OFF.
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            edit: env_truthy("LIMN_FEAT_EDIT"),
            palette: env_truthy("LIMN_FEAT_PALETTE"),
            ai: env_truthy("LIMN_FEAT_AI"),
        }
    }
}

/// Registered so the widget tree can read flags via
/// `cx.global::<FeatureFlags>()` once a wave wires that up. Not yet read
/// by any widget — see the module docs.
impl Global for FeatureFlags {}

/// Thin wrapper: read `name` from the environment and delegate the
/// truthiness decision to [`is_truthy`].
fn env_truthy(name: &str) -> bool {
    is_truthy(std::env::var(name).ok().as_deref())
}

/// Core truthiness rule, factored out of the environment so it can be
/// tested without touching process-global state. Truthy values
/// (case-insensitive) are `1`, `true`, `on`, `yes`. Anything else — and
/// `None` (absent var) — is OFF.
fn is_truthy(value: Option<&str>) -> bool {
    matches!(
        value.map(str::to_ascii_lowercase).as_deref(),
        Some("1" | "true" | "on" | "yes"),
    )
}

#[cfg(test)]
mod tests {
    use super::is_truthy;

    #[test]
    fn truthy_values_are_on() {
        for value in ["1", "true", "TRUE", "On", "yes", "YES"] {
            assert!(is_truthy(Some(value)), "{value} should be on");
        }
    }

    #[test]
    fn other_values_are_off() {
        for value in ["0", "false", "off", "no", "", "maybe"] {
            assert!(!is_truthy(Some(value)), "{value:?} should be off");
        }
    }

    #[test]
    fn absent_var_is_off() {
        assert!(!is_truthy(None));
    }
}
