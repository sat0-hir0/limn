//! gpui global wrapper for the loaded user config.
//!
//! [`limn_service::LimnConfig`] lives in `limn-service`, which is
//! `gpui`-free (ADR-0002). `gpui::Global` is a `gpui` trait, so the orphan
//! rule forbids `impl Global for LimnConfig` in the service crate. The UI
//! crate owns the binding instead via this newtype.
//!
//! Registered at startup with `cx.set_global(AppConfig(config))`, mirroring
//! [`crate::FeatureFlags`]. Wave 7 only stores it (and applies
//! `vault_path` in `main`); widgets reading `font` / `theme` from the
//! global arrive in Wave 8.

use gpui::Global;

use limn_service::LimnConfig;

/// Newtype holding the loaded [`LimnConfig`] so it can be a `gpui` global.
#[derive(Debug, Clone)]
pub struct AppConfig(pub LimnConfig);

impl Global for AppConfig {}
