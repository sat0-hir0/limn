# Visual Language — paper & ink

> **limn** /lɪm/ *(v.)* — to depict or describe; to outline with faint, delicate strokes.

This document describes Limn's visual language: the palette, type, spacing,
shape, motion, and voice conventions that any contributor — or any
design-mock-up — should follow when adding a surface, view, or icon to the
editor. It is the narrative companion to the Rust implementation in
[`crates/limn-ui/src/theme.rs`](../../crates/limn-ui/src/theme.rs).

Decision rationale lives in
[ADR-0011](../adr/0011-adopt-limn-visual-language-as-color-source-of-truth.md).
This file is the *what* and the *intent*; the ADR is the *why*.

---

## Source of truth

Three layers, in dependency order, answer the question "what colour should
this be?".

| Layer | File | Owns |
|---|---|---|
| Raw colour ramps | `crates/limn-ui/src/theme.rs` — `ColorPalette` | Hex values for the neutral ramp (`n_0` .. `n_950`), the line-blue accent (`blue_50` .. `blue_800`), approved accent alternates (sepia / graphite), and muted status hues (green / amber / red). |
| Semantic role mapping | `crates/limn-ui/src/theme.rs` — `ColorTheme` | ~30 named roles (`text_strong`, `surface_panel`, `border_hairline`, `editor_cursor`, `accent`, `positive`, …). Built via `ColorTheme::paper()` (light, default) or `ColorTheme::ink()` (dark). |
| Runtime selection | `crates/limn-ui/src/theme.rs` — `ColorTheme::from_config(limn_service::Theme)` | Resolves `LimnConfig.theme` (`Light` → paper, `Dark` → ink) into the active `ColorTheme`. |

New render code in `limn-ui` MUST source colours through `ColorTheme` rather
than reach for a hardcoded `rgb(0x...)` literal. Any new literal in a render
path is a code-review red flag (see ADR-0011).

---

## Product context

Limn is a desktop Markdown editor rendered natively with **gpui** (Rust). It
is not a web app; this design language exists to keep the native UI coherent
across views (editor, settings, palette, future panels) and to keep visual
contributions from drifting.

The stance:

| Axis | Direction |
|---|---|
| **UI** | Quiet (iA Writer–like). No handles, hover menus, or always-on toolbars. |
| **Input** | Keyboard-first. The mouse works but is never the point. Modeless (Zed lineage), not a Vim-mode toggle. |
| **AI** | Woven into writing, not bolted on. Restrained — no separate chat tab. |
| **Save** | Silent autosave. There is no "save button" concept. |
| **Data** | A folder of `.md` files is the single source of truth. No lock-in. |

**Audience:** writers who already keep a `.md` knowledge base (Obsidian,
Logseq, iA Writer, Bear) and home-row people (vim / emacs / Zed) who want AI
integration that doesn't shout.

**Inspiration:** iA Writer (thin type on a calm ground), Zed (sharp, cool,
technical), Linear (quiet panels, lines over shadow), Are.na (line,
whitespace, reading as craft). **Anti-inspiration:** loud "AI" gradients,
productivity-icon clutter (pencils, checkmarks, rockets), cute mascots,
high-saturation colour.

---

## Voice

Calm, terse, lightly poetic but never purple — close to iA Writer. No
productivity hype ("10x your output", "supercharge", "blazing-fast",
"effortless", "seamless"). No anthropomorphised assistant persona.

- **Casing:** Sentence case for UI labels and headings. The wordmark is
  lowercase `limn`. Overlines / eyebrows are UPPERCASE mono with wide
  tracking. Filenames keep their real case (`Welcome.md`).
- **Avoid:** exclamation marks, emoji, and adjectives that promise speed or
  ease.
- **Languages:** committed documentation is English (see AGENTS.md). The
  editor itself must coexist gracefully with Japanese content — the type
  stacks below fall through to IBM Plex Sans / Serif JP so EN and JP do not
  break each other inline.

---

## Palette — paper & ink

A single **cool-tinted neutral ramp** (`n_0` paper → `n_950` ink) carries
both surfaces and text. Exactly **one accent**: blue-500, a cold, low-chroma
draftsman's blue used for the cursor, focus, primary action, and links.

There is no `accent_secondary`. The single-accent constraint is enforced
structurally in `ColorTheme`: alternates (warm sepia, graphite) exist on
`ColorPalette` as approved per-product swaps but are not wired into a
default theme.

### Neutral ramp

The 14-step ramp is finer than typical 9-step ramps because the editor's
two surfaces (paper ground, sunken inputs) and four text weights (strong,
body, muted, faint) need closer-spaced neutrals than a generic system
provides. Intent of each step:

| Step | Method | Role |
|---|---|---|
| n-0 | `ColorPalette::n_0()` | Cool near-white — panels, cards |
| n-50 | `ColorPalette::n_50()` | App paper (default light surface) |
| n-100 | `ColorPalette::n_100()` | Sunken wells, input fields |
| n-150 | `ColorPalette::n_150()` | Raised / hover wash |
| n-200 | `ColorPalette::n_200()` | Hairline (light theme) |
| n-300 | `ColorPalette::n_300()` | Strong hairline (light theme) |
| n-400 | `ColorPalette::n_400()` | Faint text, disabled state |
| n-500 | `ColorPalette::n_500()` | Muted text |
| n-600 | `ColorPalette::n_600()` | Secondary text |
| n-700 | `ColorPalette::n_700()` | Body text (light theme) |
| n-800 | `ColorPalette::n_800()` | Strong text / dark panel |
| n-850 | `ColorPalette::n_850()` | Dark app paper (ink theme) |
| n-900 | `ColorPalette::n_900()` | Near-black ink |
| n-950 | `ColorPalette::n_950()` | Deepest |

### Accent — line blue

| Step | Method | Role |
|---|---|---|
| blue-50 | `ColorPalette::blue_50()` | Accent tint (paper theme) |
| blue-300 | `ColorPalette::blue_300()` | Accent in ink theme |
| blue-400 | `ColorPalette::blue_400()` | Accent (ink theme primary), accent_quiet (paper) |
| blue-500 | `ColorPalette::blue_500()` | Primary accent (paper) — cursor, focus, link, primary action |
| blue-600 | `ColorPalette::blue_600()` | Accent hover / pressed (paper) |

### Status hues — muted, never loud

| Method | Role |
|---|---|
| `ColorPalette::green_500()` | Positive |
| `ColorPalette::amber_500()` | Caution |
| `ColorPalette::red_500()` | Critical |

The corresponding tint variants live on `ColorPalette` as `green_tint()` /
`amber_tint()` / `red_tint()` for the background of an informational chip.
They are intentionally **not** promoted to `ColorTheme` semantic fields in
Wave 10; render sites that need a status-chip background should call the
palette directly for now. If a stable chip pattern emerges, a future ADR
can add `positive_tint` / `caution_tint` / `critical_tint` semantic fields
to `ColorTheme`. Saturation is intentionally low — a caution chip should
sit next to body text without fighting it.

### Two built-in themes

- **paper** (light, default) — cool paper ground, ink text. `ColorTheme::paper()`.
- **ink** (dark) — black ground, thin light text (iA Writer at night).
  `ColorTheme::ink()`.

User choice is persisted as `LimnConfig.theme` (`Light` | `Dark`) and
resolved at render-time via `ColorTheme::from_config()`. See ADR-0007 for
the config file, ADR-0010 for how settings save flips the theme live.

---

## Type

One technical sans for UI and body, one mono for code / labels / shortcuts /
the wordmark, and an optional serif reading face for long-form prose.

- **Sans (UI + body):** IBM Plex Sans
- **Mono (code, labels, shortcuts, wordmark):** IBM Plex Mono
- **Serif (optional reading face for prose):** IBM Plex Serif

Editorial scale — sizes step modestly. The editor body sits on a **68ch
measure** so paragraphs are readable without horizontal scanning.

Font family and size are user-configurable via `LimnConfig.font_family` and
`LimnConfig.font_size`; the defaults match the stack above. Japanese text
falls through to the matching IBM Plex JP family when present on the
system.

---

## Spacing and shape

- **4px grid.** All spacing snaps to 4 / 8 / 12 / 16 / 24 / 32 / 48.
- **Rectilinear.** Corner radii are small (0–8 px). The only fully-round
  shapes are avatars and status dots.
- **Editor measure:** 68ch for body content.

---

## Borders over shadows

The system separates surfaces with **hairlines** (1 px), not shadows.

- `ColorTheme::border_hairline` — default 1 px separator. In paper this is
  `n-200`; in ink it is white at ~9% alpha.
- `ColorTheme::border_strong` — when a hairline must register more firmly
  (e.g. the boundary between two panels of the same elevation).
- `ColorTheme::border_focus` / `border_accent` — line-blue for focused
  inputs and accent-bordered chips.

Shadows are few, soft, and low — reserved for things that genuinely float
above the page (command palette, dialogs, menus). They are an exception,
not a decoration.

---

## Backgrounds

Plain paper. **No gradients as decoration.** The only sanctioned gradients
are functional: e.g. a sticky title-bar protection gradient that fades the
edge of scrolling content. No textures. No imagery behind text.

---

## Motion

Quick and calm. Two durations cover almost everything:

- **120 ms fast** — hover / focus state changes
- **180 ms base** — view transitions, dialog open/close

**No bounce, no infinite loops, no parallax.** Respect
`prefers-reduced-motion`. The brand is "complete when still" — nothing
should depend on animation to be legible.

---

## Iconography

gpui draws its own glyphs natively; there is no web icon font here. When
the editor needs an icon, it ships as an SVG / native glyph in `assets/`
and is drawn through gpui's rendering path. The design mock-ups produced
during the visual-language work used Lucide as a stand-in for the eventual
native glyphs — that does **not** mean Lucide is a runtime dependency.

- **Sizes:** 14–18 px in UI.
- **Emoji:** never. They contradict the editor's voice.

---

## Editor specifics

A handful of `ColorTheme` roles exist only for the editor surface — keeping
them named (rather than reusing generic neutrals) makes "make the focused
line stand out a touch more" a one-field change instead of a hunt through
render code.

| Role | Method (paper) | Role |
|---|---|---|
| `editor_text` | `ColorPalette::n_800()` | Default body glyph colour |
| `editor_cursor` | `ColorPalette::blue_500()` | Caret colour |
| `editor_selection` | blue-500 @ 16% alpha | Range-selection background |
| `editor_syntax` | `ColorPalette::n_400()` | Markdown syntax dim (e.g. `#` of a heading) |
| `editor_focus_dim` | `ColorPalette::n_400()` | Out-of-focus block dim, when focus-mode lands |
| `editor_link` | `ColorPalette::blue_600()` | Inline link colour |

Ink-theme equivalents follow the same intent at higher alpha and lower
neutral steps — see `ColorTheme::ink()`.

### Editor canvas mapping for Wave 10-C

Wave 10-B left four hardcoded `rgb(0x...)` literals on the editor canvas
and the read-only document view. Wave 10-C will replace them with the
following semantic roles ( accessed via the active `ColorTheme`; the
exact access path — `cx.global::<ColorTheme>()` vs a per-frame resolve —
is a Wave 10-D decision ):

| Existing hardcoded | Replace with | Reason |
|---|---|---|
| `editor.rs:295` `rgb(0x00fa_f9f6)` ( bg ) | `ColorTheme::surface_app` | Editor canvas = `surface_app` ( paper or ink depending on the active theme ) |
| `editor.rs:296` `rgb(0x001a_1a1a)` ( fg ) | `ColorTheme::editor_text` | Editor body text ( the role designed for the canvas ) |
| `lib.rs:51` `rgb(0x00fa_f9f6)` ( bg ) | `ColorTheme::surface_app` | `DocumentView` ( read-only path ) shares the same canvas role |
| `lib.rs:52` `rgb(0x001a_1a1a)` ( fg ) | `ColorTheme::text_body` | `DocumentView` body text ( not `editor_text` — `DocumentView` renders read-only prose, not an editor ) |

The line numbers are approximate and will shift in the Wave 10-C diff;
the mapping by role is what matters.

### Which theme API to call

`gpui_component::Theme` and `limn_ui::ColorTheme` coexist (ADR-0011).
Knowing which API to call avoids drift between Limn-rendered surfaces
and gpui-component-rendered widgets:

| Widget / surface | Theme API |
|---|---|
| `EditorView`, `DocumentView`, `AppShell` chrome ( Limn-owned views ) | `cx.global::<ColorThemeGlobal>().0` ( Limn `ColorTheme` ) |
| `Input`, `Button`, `Switch` inside `SettingsView` ( gpui-component ports ) | `cx.theme()` ( `gpui_component::Theme` ) |
| `PaletteView` row chrome ( gpui-component `Popover` host ) | `cx.theme()` for the host frame, `ColorTheme` for Limn-rendered row content |
| Status / informational chips ( background tint ) | `ColorPalette::{green,amber,red}_tint()` directly until a `ColorTheme` role lands |
| New Limn-owned widget | `ColorTheme` ( add a role if none fits, rather than reaching for `ColorPalette` ) |

If the two APIs report different values for a role that exists in both,
that is a bug — see ADR-0011's "Negative" subsection on drift risk.

---

## Brand assets

The repo currently ships **app icons only**, under `assets/appicons/` and
the three platform SVGs at `assets/appicon-{macos,windows,linux}.svg`.

A wordmark SVG, a favicon, and a README hero were produced by the visual-
language mock-up work but **have not yet been pulled into the Limn binary's
resources**. When they land, they belong under `assets/` next to the app
icons, and this section should be updated.

Do not assume those assets exist in code review until they appear in
`assets/`.

---

## Relationship to other docs

- [ADR-0011](../adr/0011-adopt-limn-visual-language-as-color-source-of-truth.md)
  records *why* this design language is the source of truth and what the
  coexistence rule with `gpui_component::Theme` is.
- [ADR-0007](../adr/0007-user-configuration-via-toml-file.md) defines the
  `LimnConfig.theme` knob this language responds to.
- [ADR-0010](../adr/0010-settings-view-and-view-switching.md) (and its
  Wave 9 addendum) wires the settings save path so a user's theme choice
  takes effect live, not at next launch.
- [`crates/limn-ui/src/theme.rs`](../../crates/limn-ui/src/theme.rs) is the
  canonical Rust expression. Read it alongside this doc; if the two
  disagree, the code is authoritative and this doc must be updated.
