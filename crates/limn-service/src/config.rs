//! User settings persisted as TOML.
//!
//! Settings live in the platform config directory (via [`dirs`]) at
//! `limn/config.toml` and are loaded at startup, edited in the Settings
//! view, and written back on Save. Every field is `#[serde(default)]`
//! so a config file written by an older build — missing fields added
//! later — still loads, filling the gaps from [`Default`].

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Colour theme for the editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    /// Light theme (the default).
    #[default]
    Light,
    /// Dark theme.
    Dark,
}

/// User-editable settings.
///
/// Fields carry `#[serde(default)]` so partial or older config files
/// still deserialize, filling any absent field from [`Default`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// Colour theme.
    #[serde(default)]
    pub theme: Theme,
    /// Editor font family.
    #[serde(default = "default_font_family")]
    pub font_family: String,
    /// Editor font size in points.
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    /// Path to the vault folder, if the user has chosen one.
    #[serde(default)]
    pub vault_path: Option<PathBuf>,
}

fn default_font_family() -> String {
    "monospace".to_string()
}

fn default_font_size() -> f32 {
    14.0
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            font_family: default_font_family(),
            font_size: default_font_size(),
            vault_path: None,
        }
    }
}

/// Things that can go wrong loading or saving [`Config`].
#[derive(Debug)]
pub enum ConfigError {
    /// The config file (or its parent directory) could not be read or written.
    Io(io::Error),
    /// The config file exists but is not valid TOML for [`Config`].
    Parse(toml::de::Error),
    /// The config could not be serialized to TOML.
    Serialize(toml::ser::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Parse(e) => write!(f, "invalid config TOML: {e}"),
            Self::Serialize(e) => write!(f, "could not serialize config: {e}"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Parse(e) => Some(e),
            Self::Serialize(e) => Some(e),
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        Self::Parse(e)
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(e: toml::ser::Error) -> Self {
        Self::Serialize(e)
    }
}

impl Config {
    /// The resolved path to the user's config file:
    /// `<config_dir>/limn/config.toml`.
    ///
    /// On Windows this resolves under `%APPDATA%`, on Linux under
    /// `$XDG_CONFIG_HOME` (or `~/.config`), and on macOS under
    /// `~/Library/Application Support`. If the platform config directory
    /// cannot be determined, falls back to a relative `limn/config.toml`.
    #[must_use]
    pub fn config_path() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("limn").join("config.toml")
    }

    /// Load settings from the user's config file.
    ///
    /// A missing file is not an error — first run is normal — and yields
    /// [`Config::default`].
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Parse`] if the file exists but is not valid
    /// TOML, or [`ConfigError::Io`] on any read error other than
    /// "not found".
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_from(&Self::config_path())
    }

    /// Load settings from an explicit path.
    ///
    /// A missing file yields [`Config::default`]; see [`Config::load`].
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Parse`] if the file exists but is not valid
    /// TOML, or [`ConfigError::Io`] on any read error other than
    /// "not found".
    pub fn load_from(path: &Path) -> Result<Self, ConfigError> {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(ConfigError::Io(e)),
        };
        Ok(toml::from_str(&text)?)
    }

    /// Save settings to the user's config file, creating parent
    /// directories as needed.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Serialize`] if the config cannot be
    /// serialized, or [`ConfigError::Io`] on any write error.
    pub fn save(&self) -> Result<(), ConfigError> {
        self.save_to(&Self::config_path())
    }

    /// Save settings to an explicit path, creating parent directories as
    /// needed.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Serialize`] if the config cannot be
    /// serialized, or [`ConfigError::Io`] on any write error.
    pub fn save_to(&self, path: &Path) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self)?;
        fs::write(path, text)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let original = Config::default();

        original.save_to(&path).unwrap();
        let loaded = Config::load_from(&path).unwrap();

        assert_eq!(original, loaded);
    }

    #[test]
    fn non_default_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let original = Config {
            theme: Theme::Dark,
            font_family: "Fira Code".to_string(),
            font_size: 16.5,
            vault_path: Some(PathBuf::from("/home/user/notes")),
        };

        original.save_to(&path).unwrap();
        let loaded = Config::load_from(&path).unwrap();

        assert_eq!(original, loaded);
    }

    #[test]
    fn missing_file_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("does-not-exist.toml");

        let loaded = Config::load_from(&path).unwrap();

        assert_eq!(loaded, Config::default());
    }

    #[test]
    fn partial_file_fills_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(&path, "theme = \"dark\"\n").unwrap();

        let loaded = Config::load_from(&path).unwrap();

        assert_eq!(loaded.theme, Theme::Dark);
        assert_eq!(loaded.font_family, default_font_family());
        assert!((loaded.font_size - default_font_size()).abs() < f32::EPSILON);
        assert_eq!(loaded.vault_path, None);
    }

    #[test]
    fn invalid_toml_surfaces_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(&path, "this is = not = valid toml\n").unwrap();

        let err = Config::load_from(&path).unwrap_err();

        assert!(matches!(err, ConfigError::Parse(_)));
    }

    #[test]
    fn config_path_ends_with_expected_tail() {
        let path = Config::config_path();
        assert!(path.ends_with(Path::new("limn").join("config.toml")));
    }
}
