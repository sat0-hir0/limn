//! Color theme for the Limn design system.
//!
//! Two built-in themes are provided:
//! - [`Theme::paper()`] — light (default)
//! - [`Theme::ink()`] — dark
//!
//! [`Palette`] exposes the raw color ramp. [`Theme`] maps palette colors
//! to semantic roles (surface, text, accent, editor, status).
//! Design source of truth: Limn design system (see `docs/design/`).

use gpui::{rgb, rgba, Rgba};

/// Raw palette — the cool-tinted neutral ramp, the single accent,
/// approved accent alternates, and muted status hues. Theme-independent.
#[derive(Debug, Clone, Copy)]
pub struct Palette;

impl Palette {
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

/// Semantic color roles. Build with `Theme::paper()` (light, default)
/// or `Theme::ink()` (dark). Field names mirror the CSS `--*` aliases.
#[derive(Clone, Copy, Debug)]
pub struct Theme {
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

impl Theme {
    /// PAPER — light, default. Cool paper ground, ink text.
    #[must_use]
    pub fn paper() -> Self {
        Self {
            text_strong: Palette::n_900(),
            text_body: Palette::n_700(),
            text_muted: Palette::n_500(),
            text_faint: Palette::n_400(),
            text_accent: Palette::blue_600(),
            text_on_accent: Palette::n_0(),
            text_inverse: Palette::n_50(),

            surface_app: Palette::n_50(),
            surface_panel: Palette::n_0(),
            surface_raised: Palette::n_0(),
            surface_sunken: Palette::n_100(),
            surface_hover: Palette::n_100(),
            surface_active: Palette::n_150(),
            surface_overlay: rgba(0x1518_1b52), // ink @ 0.32

            border_hairline: Palette::n_200(),
            border_strong: Palette::n_300(),
            border_accent: Palette::blue_500(),
            border_focus: Palette::blue_500(),

            accent: Palette::blue_500(),
            accent_hover: Palette::blue_600(),
            accent_quiet: Palette::blue_400(),
            accent_contrast: Palette::n_0(),
            accent_tint: Palette::blue_50(),
            accent_line: Palette::blue_500(),

            editor_text: Palette::n_800(),
            editor_cursor: Palette::blue_500(),
            editor_selection: rgba(0x3e6d_b529), // blue-500 @ 0.16
            editor_syntax: Palette::n_400(),
            editor_focus_dim: Palette::n_400(),
            editor_link: Palette::blue_600(),

            positive: Palette::green_500(),
            caution: Palette::amber_500(),
            critical: Palette::red_500(),
        }
    }

    /// INK — dark. Black ground, thin light text (iA Writer at night).
    #[must_use]
    pub fn ink() -> Self {
        Self {
            text_strong: Palette::n_50(),
            text_body: rgb(0x00c8_ccce),
            text_muted: rgb(0x008d_9398),
            text_faint: rgb(0x0063_6a6f),
            text_accent: Palette::blue_300(),
            text_on_accent: Palette::n_0(),
            text_inverse: Palette::n_900(),

            surface_app: Palette::n_850(),
            surface_panel: Palette::n_800(),
            surface_raised: rgb(0x0024_2a2f),
            surface_sunken: Palette::n_900(),
            surface_hover: rgba(0xffff_ff0a),   // white @ 0.04
            surface_active: rgba(0xffff_ff12),  // white @ 0.07
            surface_overlay: rgba(0x0809_0a8c), // near-black @ 0.55

            border_hairline: rgba(0xffff_ff17), // white @ 0.09
            border_strong: rgba(0xffff_ff29),   // white @ 0.16
            border_accent: Palette::blue_400(),
            border_focus: Palette::blue_400(),

            accent: Palette::blue_400(),
            accent_hover: Palette::blue_300(),
            accent_quiet: Palette::blue_500(),
            accent_contrast: Palette::n_900(),
            accent_tint: rgba(0x5c84_c229), // blue-400 @ 0.16
            accent_line: Palette::blue_400(),

            editor_text: rgb(0x00d4_d8da),
            editor_cursor: Palette::blue_400(),
            editor_selection: rgba(0x5c84_c242), // blue-400 @ 0.26
            editor_syntax: rgb(0x005c_6368),
            editor_focus_dim: rgb(0x004d_5358),
            editor_link: Palette::blue_300(),

            positive: rgb(0x006f_9e7c),
            caution: rgb(0x00c0_a261),
            critical: rgb(0x00c4_7b73),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::paper()
    }
}

#[cfg(test)]
mod tests {
    use gpui::rgb;

    use super::*;

    #[test]
    fn palette_n50_hex() {
        assert_eq!(Palette::n_50(), rgb(0x00f0_f3f4));
    }

    #[test]
    fn palette_blue500_hex() {
        assert_eq!(Palette::blue_500(), rgb(0x003e_6db5));
    }

    #[test]
    fn paper_surface_app() {
        assert_eq!(Theme::paper().surface_app, Palette::n_50());
    }

    #[test]
    fn ink_surface_app() {
        assert_eq!(Theme::ink().surface_app, Palette::n_850());
    }

    #[test]
    fn paper_accent() {
        assert_eq!(Theme::paper().accent, Palette::blue_500());
    }

    #[test]
    fn ink_accent() {
        assert_eq!(Theme::ink().accent, Palette::blue_400());
    }

    #[test]
    fn paper_editor_cursor() {
        assert_eq!(Theme::paper().editor_cursor, Palette::blue_500());
    }

    #[test]
    fn ink_editor_cursor() {
        assert_eq!(Theme::ink().editor_cursor, Palette::blue_400());
    }

    #[test]
    fn default_equals_paper() {
        let default = Theme::default();
        let paper = Theme::paper();
        assert_eq!(default.surface_app, paper.surface_app);
        assert_eq!(default.accent, paper.accent);
        assert_eq!(default.editor_cursor, paper.editor_cursor);
    }
}
