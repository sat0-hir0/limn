//! Color theme for the Limn design system.
//!
//! Two built-in themes are provided:
//! - [`ColorTheme::paper()`] — light (default)
//! - [`ColorTheme::ink()`] — dark
//!
//! Use [`ColorTheme::from_config()`] to resolve the active theme from the
//! user's persisted [`limn_service::Theme`] choice.
//!
//! [`ColorPalette`] exposes the raw color ramp. [`ColorTheme`] maps palette
//! colors to semantic roles (surface, text, accent, editor, status).
//! Design source of truth: Limn design system (see `docs/design/`).

use gpui::{rgb, rgba, Rgba};

/// Raw palette — the cool-tinted neutral ramp, the single accent,
/// approved accent alternates, and muted status hues. Theme-independent.
#[derive(Debug, Clone, Copy)]
pub struct ColorPalette;

impl ColorPalette {
    // ---- Neutral ramp — cool paper → ink ----
    #[must_use]
    pub fn n_0() -> Rgba {
        rgb(0x00f8_fafb)
    } // cool near-white — panels, cards
    #[must_use]
    pub fn n_50() -> Rgba {
        rgb(0x00f0_f3f4)
    } // app paper
    #[must_use]
    pub fn n_100() -> Rgba {
        rgb(0x00e9_edee)
    } // sunken wells, inputs
    #[must_use]
    pub fn n_150() -> Rgba {
        rgb(0x00e1_e7e8)
    } // raised / hover wash
    #[must_use]
    pub fn n_200() -> Rgba {
        rgb(0x00d3_dadb)
    } // hairline (light)
    #[must_use]
    pub fn n_300() -> Rgba {
        rgb(0x00bb_c3c4)
    } // strong hairline (light)
    #[must_use]
    pub fn n_400() -> Rgba {
        rgb(0x009b_9e9e)
    } // faint text / disabled
    #[must_use]
    pub fn n_500() -> Rgba {
        rgb(0x006d_7378)
    } // muted text
    #[must_use]
    pub fn n_600() -> Rgba {
        rgb(0x004a_5056)
    } // secondary text
    #[must_use]
    pub fn n_700() -> Rgba {
        rgb(0x0034_3a40)
    } // body text
    #[must_use]
    pub fn n_800() -> Rgba {
        rgb(0x0021_262b)
    } // strong text / dark panel
    #[must_use]
    pub fn n_850() -> Rgba {
        rgb(0x001a_1e22)
    } // dark app paper
    #[must_use]
    pub fn n_900() -> Rgba {
        rgb(0x0015_181b)
    } // near-black ink
    #[must_use]
    pub fn n_950() -> Rgba {
        rgb(0x000f_1113)
    } // deepest

    // ---- Accent — "line blue" (cold draftsman blue) ----
    #[must_use]
    pub fn blue_50() -> Rgba {
        rgb(0x00ee_f2f8)
    }
    #[must_use]
    pub fn blue_100() -> Rgba {
        rgb(0x00db_e5f1)
    }
    #[must_use]
    pub fn blue_200() -> Rgba {
        rgb(0x00b4_c8e3)
    }
    #[must_use]
    pub fn blue_300() -> Rgba {
        rgb(0x0084_a3d0)
    }
    #[must_use]
    pub fn blue_400() -> Rgba {
        rgb(0x005c_84c2)
    }
    #[must_use]
    pub fn blue_500() -> Rgba {
        rgb(0x003e_6db5)
    } // primary accent
    #[must_use]
    pub fn blue_600() -> Rgba {
        rgb(0x0033_5c9c)
    } // hover / pressed
    #[must_use]
    pub fn blue_700() -> Rgba {
        rgb(0x002a_4d82)
    }
    #[must_use]
    pub fn blue_800() -> Rgba {
        rgb(0x0023_3e68)
    }

    // ---- Approved accent alternates (not the default) ----
    #[must_use]
    pub fn alt_sepia_500() -> Rgba {
        rgb(0x009a_7b4f)
    }
    #[must_use]
    pub fn alt_sepia_tint() -> Rgba {
        rgb(0x00f3_ede2)
    }
    #[must_use]
    pub fn alt_graphite_500() -> Rgba {
        rgb(0x004b_5258)
    }
    #[must_use]
    pub fn alt_graphite_tint() -> Rgba {
        rgb(0x00ec_ecea)
    }

    // ---- Status — muted, never loud ----
    #[must_use]
    pub fn green_500() -> Rgba {
        rgb(0x004a_7c59)
    } // positive
    #[must_use]
    pub fn green_tint() -> Rgba {
        rgb(0x00ea_f1ec)
    }
    #[must_use]
    pub fn amber_500() -> Rgba {
        rgb(0x009c_7a36)
    } // caution
    #[must_use]
    pub fn amber_tint() -> Rgba {
        rgb(0x00f4_eedf)
    }
    #[must_use]
    pub fn red_500() -> Rgba {
        rgb(0x00a4_524b)
    } // critical
    #[must_use]
    pub fn red_tint() -> Rgba {
        rgb(0x00f5_e9e7)
    }
}

/// Semantic color roles. Build with `ColorTheme::paper()` (light, default)
/// or `ColorTheme::ink()` (dark). Field names mirror the CSS `--*` aliases.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorTheme {
    // Text
    pub text_strong: Rgba,
    pub text_body: Rgba,
    pub text_muted: Rgba,
    pub text_faint: Rgba,
    pub text_accent: Rgba,
    pub text_on_accent: Rgba,
    pub text_inverse: Rgba,

    // Surfaces
    pub surface_app: Rgba,
    pub surface_panel: Rgba,
    pub surface_raised: Rgba,
    pub surface_sunken: Rgba,
    pub surface_hover: Rgba,
    pub surface_active: Rgba,
    pub surface_overlay: Rgba,

    // Borders — hairlines, not shadows
    pub border_hairline: Rgba,
    pub border_strong: Rgba,
    pub border_accent: Rgba,
    pub border_focus: Rgba,

    // Accent roles
    pub accent: Rgba,
    pub accent_hover: Rgba,
    pub accent_quiet: Rgba,
    pub accent_contrast: Rgba,
    pub accent_tint: Rgba,
    pub accent_line: Rgba,

    // Editor specifics
    pub editor_text: Rgba,
    pub editor_cursor: Rgba,
    pub editor_selection: Rgba,
    pub editor_syntax: Rgba,
    pub editor_focus_dim: Rgba,
    pub editor_link: Rgba,

    // Status roles
    pub positive: Rgba,
    pub caution: Rgba,
    pub critical: Rgba,
}

impl ColorTheme {
    /// PAPER — light, default. Cool paper ground, ink text.
    #[must_use]
    pub fn paper() -> Self {
        Self {
            text_strong: ColorPalette::n_900(),
            text_body: ColorPalette::n_700(),
            text_muted: ColorPalette::n_500(),
            text_faint: ColorPalette::n_400(),
            text_accent: ColorPalette::blue_600(),
            text_on_accent: ColorPalette::n_0(),
            text_inverse: ColorPalette::n_50(),

            surface_app: ColorPalette::n_50(),
            surface_panel: ColorPalette::n_0(),
            surface_raised: ColorPalette::n_0(),
            surface_sunken: ColorPalette::n_100(),
            surface_hover: ColorPalette::n_100(),
            surface_active: ColorPalette::n_150(),
            surface_overlay: rgba(0x1518_1b52), // ink @ 0.32

            border_hairline: ColorPalette::n_200(),
            border_strong: ColorPalette::n_300(),
            border_accent: ColorPalette::blue_500(),
            border_focus: ColorPalette::blue_500(),

            accent: ColorPalette::blue_500(),
            accent_hover: ColorPalette::blue_600(),
            accent_quiet: ColorPalette::blue_400(),
            accent_contrast: ColorPalette::n_0(),
            accent_tint: ColorPalette::blue_50(),
            accent_line: ColorPalette::blue_500(),

            editor_text: ColorPalette::n_800(),
            editor_cursor: ColorPalette::blue_500(),
            editor_selection: rgba(0x3e6d_b529), // blue-500 @ 0.16
            editor_syntax: ColorPalette::n_400(),
            editor_focus_dim: ColorPalette::n_400(),
            editor_link: ColorPalette::blue_600(),

            positive: ColorPalette::green_500(),
            caution: ColorPalette::amber_500(),
            critical: ColorPalette::red_500(),
        }
    }

    /// INK — dark. Black ground, thin light text (iA Writer at night).
    #[must_use]
    pub fn ink() -> Self {
        Self {
            text_strong: ColorPalette::n_50(),
            text_body: rgb(0x00c8_ccce),
            text_muted: rgb(0x008d_9398),
            text_faint: rgb(0x0063_6a6f),
            text_accent: ColorPalette::blue_300(),
            text_on_accent: ColorPalette::n_0(),
            text_inverse: ColorPalette::n_900(),

            surface_app: ColorPalette::n_850(),
            surface_panel: ColorPalette::n_800(),
            surface_raised: rgb(0x0024_2a2f),
            surface_sunken: ColorPalette::n_900(),
            surface_hover: rgba(0xffff_ff0a),   // white @ 0.04
            surface_active: rgba(0xffff_ff12),  // white @ 0.07
            surface_overlay: rgba(0x0809_0a8c), // near-black @ 0.55

            border_hairline: rgba(0xffff_ff17), // white @ 0.09
            border_strong: rgba(0xffff_ff29),   // white @ 0.16
            border_accent: ColorPalette::blue_400(),
            border_focus: ColorPalette::blue_400(),

            accent: ColorPalette::blue_400(),
            accent_hover: ColorPalette::blue_300(),
            accent_quiet: ColorPalette::blue_500(),
            accent_contrast: ColorPalette::n_900(),
            accent_tint: rgba(0x5c84_c229), // blue-400 @ 0.16
            accent_line: ColorPalette::blue_400(),

            editor_text: rgb(0x00d4_d8da),
            editor_cursor: ColorPalette::blue_400(),
            editor_selection: rgba(0x5c84_c242), // blue-400 @ 0.26
            editor_syntax: rgb(0x005c_6368),
            editor_focus_dim: rgb(0x004d_5358),
            editor_link: ColorPalette::blue_300(),

            positive: rgb(0x006f_9e7c),
            caution: rgb(0x00c0_a261),
            critical: rgb(0x00c4_7b73),
        }
    }

    /// Build a [`ColorTheme`] from the user's persisted theme choice.
    ///
    /// Render code should call this with the active [`limn_service::Theme`]
    /// (sourced from [`limn_service::LimnConfig::theme`]) rather than
    /// matching on the enum at every render site.
    #[must_use]
    pub fn from_config(theme: limn_service::Theme) -> Self {
        match theme {
            limn_service::Theme::Light => Self::paper(),
            limn_service::Theme::Dark => Self::ink(),
        }
    }
}

impl Default for ColorTheme {
    fn default() -> Self {
        Self::paper()
    }
}

#[cfg(test)]
mod tests {
    // Direct f32 field comparisons below are exact: both sides come from
    // the same deterministic `u8 -> f32 / 255.0` computation that gpui's
    // `rgba()` macro performs, so bitwise equality holds.
    #![allow(clippy::float_cmp)]

    use gpui::rgb;

    use super::*;

    #[test]
    fn palette_n50_hex() {
        assert_eq!(ColorPalette::n_50(), rgb(0x00f0_f3f4));
    }

    #[test]
    fn palette_blue500_hex() {
        assert_eq!(ColorPalette::blue_500(), rgb(0x003e_6db5));
    }

    #[test]
    fn paper_surface_app() {
        assert_eq!(ColorTheme::paper().surface_app, ColorPalette::n_50());
    }

    #[test]
    fn ink_surface_app() {
        assert_eq!(ColorTheme::ink().surface_app, ColorPalette::n_850());
    }

    #[test]
    fn paper_accent() {
        assert_eq!(ColorTheme::paper().accent, ColorPalette::blue_500());
    }

    #[test]
    fn ink_accent() {
        assert_eq!(ColorTheme::ink().accent, ColorPalette::blue_400());
    }

    #[test]
    fn paper_editor_cursor() {
        assert_eq!(ColorTheme::paper().editor_cursor, ColorPalette::blue_500());
    }

    #[test]
    fn ink_editor_cursor() {
        assert_eq!(ColorTheme::ink().editor_cursor, ColorPalette::blue_400());
    }

    #[test]
    fn default_equals_paper() {
        assert_eq!(ColorTheme::default(), ColorTheme::paper());
    }

    #[test]
    fn paper_and_ink_differ() {
        assert_ne!(ColorTheme::paper(), ColorTheme::ink());
    }

    #[test]
    fn paper_surface_overlay_is_ink_with_alpha() {
        let overlay = ColorTheme::paper().surface_overlay;
        // ink @ 0.32 — see doc comment in paper() impl
        assert_eq!(overlay.r, f32::from(0x15_u8) / 255.0);
        assert_eq!(overlay.g, f32::from(0x18_u8) / 255.0);
        assert_eq!(overlay.b, f32::from(0x1b_u8) / 255.0);
        assert_eq!(overlay.a, f32::from(0x52_u8) / 255.0);
    }

    #[test]
    fn ink_surface_overlay_is_near_black_with_alpha() {
        let overlay = ColorTheme::ink().surface_overlay;
        // near-black @ 0.55
        assert_eq!(overlay.r, f32::from(0x08_u8) / 255.0);
        assert_eq!(overlay.g, f32::from(0x09_u8) / 255.0);
        assert_eq!(overlay.b, f32::from(0x0a_u8) / 255.0);
        assert_eq!(overlay.a, f32::from(0x8c_u8) / 255.0);
    }

    #[test]
    fn ink_border_hairline_is_white_at_low_alpha() {
        let border = ColorTheme::ink().border_hairline;
        // white @ 0.09
        assert_eq!(border.r, 1.0);
        assert_eq!(border.g, 1.0);
        assert_eq!(border.b, 1.0);
        assert_eq!(border.a, f32::from(0x17_u8) / 255.0);
    }

    #[test]
    fn paper_editor_selection_is_blue500_with_alpha() {
        let sel = ColorTheme::paper().editor_selection;
        // blue-500 @ 0.16
        assert_eq!(sel.r, f32::from(0x3e_u8) / 255.0);
        assert_eq!(sel.g, f32::from(0x6d_u8) / 255.0);
        assert_eq!(sel.b, f32::from(0xb5_u8) / 255.0);
        assert_eq!(sel.a, f32::from(0x29_u8) / 255.0);
    }

    #[test]
    fn from_config_maps_light_to_paper() {
        assert_eq!(
            ColorTheme::from_config(limn_service::Theme::Light),
            ColorTheme::paper()
        );
    }

    #[test]
    fn from_config_maps_dark_to_ink() {
        assert_eq!(
            ColorTheme::from_config(limn_service::Theme::Dark),
            ColorTheme::ink()
        );
    }
}
