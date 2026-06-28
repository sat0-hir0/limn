//! User configuration: read/write `~/.config/limn/config.toml`.
//!
//! Wave 7 introduces the minimal config mechanism — `font`, `theme`, and
//! `vault_path` — plus the load/save plumbing. The file lives at a fixed
//! literal path (`<home>/.config/limn/config.toml`) on every OS, derived
//! from [`dirs::home_dir`] (see ADR-0007 for why we use that literal path
//! rather than an XDG / platform-specific config dir).
//!
//! Layering: this type lives in `limn-service` (ADR-0002) and stays
//! `gpui`-free; the `gpui::Global` newtype wrapper that lets the UI hold a
//! loaded config lives in `limn-ui`.
//!
//! Wave 7 scope: the config is loaded at startup and `vault_path` is
//! applied (it picks the vault root when no path argument is given).
//! `font` / `theme` are loaded and carried but not yet applied to
//! rendering — that, plus a settings UI, is Wave 8.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Top-level user configuration.
///
/// # Forward compatibility
///
/// The struct carries `#[serde(default)]` and deliberately does **not**
/// carry `#[serde(deny_unknown_fields)]`. Together these give two-way
/// forward compatibility:
///
/// - A *missing* field (an older file written before the field existed)
///   falls back to its [`Default`], so new fields never break old files.
/// - An *unknown* field (a file written by a newer build) is silently
///   ignored rather than rejected, so a newer file never breaks an older
///   build. (The unknown field is dropped on the next save; round-tripping
///   unknown keys is out of scope for Wave 7.)
///
/// Do not add `deny_unknown_fields` here — it would turn a forward-compat
/// read into a hard parse error (see ADR-0007).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LimnConfig {
    pub font: FontConfig,
    pub theme: Theme,
    pub vault_path: Option<PathBuf>,
}

/// Editor font preferences.
///
/// Carries `#[serde(default)]` for the same per-field forward-compat
/// reason as [`LimnConfig`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FontConfig {
    pub family: String,
    pub size: u16,
}

impl Default for FontConfig {
    fn default() -> Self {
        // Empty family = "let the renderer pick its default font". The
        // numeric size has a concrete default so a partial config still
        // yields a usable value. (Neither is applied to rendering until
        // Wave 8.)
        Self {
            family: String::new(),
            size: 14,
        }
    }
}

/// Color theme. `Dark` is the default to match the editor's current
/// look.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    #[default]
    Dark,
}

/// Things that can go wrong *saving* config. Modeled on
/// [`crate::vault::OpenError`].
///
/// Loading never surfaces an error (it always falls back to defaults — see
/// [`LimnConfig::load`]); only saving returns a `Result`.
#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Serialize(toml::ser::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Serialize(e) => write!(f, "config serialization error: {e}"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Serialize(e) => Some(e),
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(e: toml::ser::Error) -> Self {
        Self::Serialize(e)
    }
}

impl LimnConfig {
    /// The fixed config-file location: `<home>/.config/limn/config.toml`.
    ///
    /// Returns `None` only when the home directory can't be resolved (a
    /// degenerate environment). The literal `.config/limn` segment is used
    /// on every OS by design (ADR-0007), not the platform config dir.
    #[must_use]
    pub fn config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".config").join("limn").join("config.toml"))
    }

    /// Load the user config, never failing.
    ///
    /// Resolution and fallback rules:
    ///
    /// - Home dir unresolvable → defaults (cannot locate the file).
    /// - File missing → write the defaults out (best effort) and return
    ///   defaults, so a first run leaves a self-documenting file on disk.
    /// - File present but unparseable → return defaults and **leave the
    ///   broken file untouched** (never overwrite the user's file on a
    ///   parse error; they may want to fix it by hand).
    ///
    /// Every fallback path logs to `stderr` so the reason is observable
    /// without changing the return type. Startup must not be blocked by a
    /// bad config (see ADR-0007), hence `Self` rather than `Result`.
    #[must_use]
    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            eprintln!("limn-service: cannot resolve home dir; using default config");
            return Self::default();
        };
        Self::load_from(&path)
    }

    /// Load from an explicit path. Same fallback rules as [`load`], but
    /// without the process-global home lookup, so tests can drive it with
    /// a `tempfile` path.
    ///
    /// [`load`]: LimnConfig::load
    #[must_use]
    pub fn load_from(path: &Path) -> Self {
        match fs::read_to_string(path) {
            Ok(text) => match toml::from_str(&text) {
                Ok(config) => config,
                Err(e) => {
                    // Parse failure: fall back to defaults and DO NOT
                    // overwrite — preserve the broken file as-is.
                    eprintln!(
                        "limn-service: failed to parse {}: {e}; using default config (file left untouched)",
                        path.display()
                    );
                    Self::default()
                }
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                // First run: materialize the defaults so the file is
                // discoverable and editable. A write failure here is
                // non-fatal — we still return usable defaults.
                let config = Self::default();
                if let Err(write_err) = config.save_to(path) {
                    eprintln!(
                        "limn-service: failed to write default config to {}: {write_err}",
                        path.display()
                    );
                }
                config
            }
            Err(e) => {
                eprintln!(
                    "limn-service: failed to read {}: {e}; using default config",
                    path.display()
                );
                Self::default()
            }
        }
    }

    /// Save the config to its canonical [`config_path`].
    ///
    /// [`config_path`]: LimnConfig::config_path
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Io`] if the home dir can't be resolved, the
    /// parent dir can't be created, or the write/rename fails;
    /// [`ConfigError::Serialize`] if the config can't be serialized.
    pub fn save(&self) -> Result<(), ConfigError> {
        let path = Self::config_path().ok_or_else(|| {
            ConfigError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "cannot resolve home directory",
            ))
        })?;
        self.save_to(&path)
    }

    /// Save to an explicit path, creating any missing parent directories.
    ///
    /// The write is atomic from a reader's point of view: it goes to a
    /// sibling temp file first, then renames over `path` (the same
    /// reader-atomicity pattern as [`crate::vault::Vault::save_raw`], per
    /// ADR-0005). Config writes are rare and single-writer, so the
    /// per-write PID+counter temp naming from `save_raw` is unnecessary
    /// here — a fixed `.limn-tmp` suffix suffices.
    ///
    /// # Errors
    ///
    /// See [`save`].
    ///
    /// [`save`]: LimnConfig::save
    pub fn save_to(&self, path: &Path) -> Result<(), ConfigError> {
        let text = toml::to_string_pretty(self)?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Co-locate the temp file with the destination so `rename` is an
        // intra-filesystem (atomic) move.
        let mut tmp = path.as_os_str().to_owned();
        tmp.push(".limn-tmp");
        let tmp = PathBuf::from(tmp);

        if let Err(e) = fs::write(&tmp, text.as_bytes()) {
            let _ = fs::remove_file(&tmp);
            return Err(e.into());
        }
        if let Err(e) = fs::rename(&tmp, path) {
            let _ = fs::remove_file(&tmp);
            return Err(e.into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{FontConfig, LimnConfig, Theme};

    #[test]
    fn round_trip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");

        let config = LimnConfig {
            font: FontConfig {
                family: "Fira Code".to_string(),
                size: 16,
            },
            theme: Theme::Light,
            vault_path: Some(dir.path().join("notes")),
        };
        config.save_to(&path).expect("save");

        let loaded = LimnConfig::load_from(&path);
        assert_eq!(loaded, config);
    }

    #[test]
    fn default_when_missing() {
        let dir = tempfile::tempdir().expect("tempdir");
        // Point at a file that does not exist yet under a dir that does.
        let path = dir.path().join("config.toml");

        let loaded = LimnConfig::load_from(&path);
        assert_eq!(loaded, LimnConfig::default());
        // load_from on a missing file materializes the defaults.
        assert!(
            path.exists(),
            "missing file should be created with defaults"
        );

        let reloaded = LimnConfig::load_from(&path);
        assert_eq!(reloaded, LimnConfig::default());
    }

    #[test]
    fn default_on_broken_toml() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let broken = "this is = = not valid toml [[[";
        std::fs::write(&path, broken).expect("write broken");

        let loaded = LimnConfig::load_from(&path);
        assert_eq!(loaded, LimnConfig::default());

        // The broken file must be preserved verbatim, not overwritten.
        let on_disk = std::fs::read_to_string(&path).expect("read back");
        assert_eq!(on_disk, broken, "broken config must be left untouched");
    }

    #[test]
    fn forward_compat_unknown_field() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        // A file written by a hypothetical newer build with a field this
        // build doesn't know about. It must still parse.
        std::fs::write(&path, "theme = \"dark\"\nfuture_field = 42\n").expect("write");

        let loaded = LimnConfig::load_from(&path);
        assert_eq!(loaded.theme, Theme::Dark);
    }

    #[test]
    fn missing_field_uses_default() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        // Only `theme` present; `font` and `vault_path` omitted entirely.
        std::fs::write(&path, "theme = \"light\"\n").expect("write");

        let loaded = LimnConfig::load_from(&path);
        assert_eq!(loaded.theme, Theme::Light);
        assert_eq!(loaded.font, FontConfig::default());
        assert_eq!(loaded.vault_path, None);
    }

    #[test]
    fn atomic_save_creates_parent() {
        let dir = tempfile::tempdir().expect("tempdir");
        // A nested path whose parent dirs do not exist yet.
        let path = dir.path().join("nested").join("deeper").join("config.toml");
        assert!(!path.parent().unwrap().exists());

        let config = LimnConfig {
            theme: Theme::Light,
            ..LimnConfig::default()
        };
        config.save_to(&path).expect("save into uncreated dirs");

        assert!(path.exists(), "parent dirs should be created");
        let loaded = LimnConfig::load_from(&path);
        assert_eq!(loaded, config);
    }
}
