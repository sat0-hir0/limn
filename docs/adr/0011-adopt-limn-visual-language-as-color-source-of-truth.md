# ADR-0011: Adopt Limn visual language ( paper & ink ) as the source of truth for color

- **Status**: Proposed
- **Date**: 2026-06-30
- **Deciders**: sat0-hir0

---

## Context

Until Wave 10, Limn had **no documented color palette and no design
tokens**. Surfaces grew opportunistically:

- `crates/limn-ui/src/lib.rs` (`DocumentView::render`) hardcoded
  `rgb(0x00fa_f9f6)` for background and `rgb(0x001a_1a1a)` for foreground —
  literals that date back to Wave 1 and never had a name.
- `crates/limn-ui/src/editor.rs` did the same for the editing shell.
- `SettingsView` (Wave 8, ADR-0010) and `PaletteView` adopted
  `gpui_component::ActiveTheme::theme()` which exposes a generic
  `gpui_component::Theme` struct. That covered components imported from
  gpui-component (`Input`, `Switch`, `Button`) but not the editor surface
  itself.

Wave 9 (ADR-0010 addendum) wired user theme changes to
`gpui_component::Theme::change(..)` so the global flips at save time and
`cx.refresh_windows()` redraws. The addendum's "theme reactivity" was
real, but only for surfaces that already read `cx.theme()` — the editor
still rendered its hardcoded literals, so a user switching to dark mode
saw gpui-component surfaces flip while the editing canvas stayed cream.

The existing hardcoded literals are `rgb(0x00fa_f9f6)` ( a warm
off-white, dating back to Wave 1 ) for background and `rgb(0x001a_1a1a)`
( near-black ) for text. The design system's equivalents are
`ColorPalette::n_50()` = `rgb(0x00f0_f3f4)` ( a cool-tinted paper ) and
`ColorTheme::paper().text_body` = `ColorPalette::n_700()` =
`rgb(0x0034_3a40)`. Adopting the palette is therefore a deliberate
visual change in Wave 10-C, not a behaviour-preserving refactor — the
editor's paper surface acquires the cool tint that the visual language
calls for, and body text moves from near-black to a slightly softer
ink. The change is intentional; this ADR records it so that Wave 10-C's
diff does not look like a regression in review.

A design system was then produced externally (the claude.ai/design
project) covering:

- A 14-step cool neutral ramp (`n-0` .. `n-950`)
- A single accent ( `blue-50` .. `blue-800` "line blue" )
- Two paper-and-ink theme variants
- Typographic tokens ( IBM Plex Sans / Mono / Serif )
- Spacing and shape tokens ( 4 px grid, small radii, hairlines over
  shadows )
- A voice and iconography stance ( quiet, keyboard-first, no emoji )

That work was a complete design system but produced for a React / web
target. This ADR adopts it as Limn's source of truth for color, with
`crates/limn-ui/src/theme.rs` (Wave 10-A) as the canonical Rust
expression.

---

## Decision

We adopt the **Limn visual language** documented in
[`docs/design/visual-language.md`](../design/visual-language.md) as
authoritative for color, type intent, spacing intent, and brand voice,
and we anchor color implementation in `crates/limn-ui/src/theme.rs`.

Concretely:

1. **`docs/design/visual-language.md` is the narrative source of truth.**
   New surfaces are designed against the roles described there. When the
   doc and the Rust code disagree, the code is authoritative and the doc
   must be updated.

2. **`crates/limn-ui/src/theme.rs` is the Rust expression of the color
   tokens.**
   - `ColorPalette` exposes the raw ramp: neutral `n_0` .. `n_950`,
     accent `blue_50` .. `blue_800`, alt sepia / graphite, status
     green / amber / red.
   - `ColorTheme` maps the palette to ~30 semantic roles
     (`text_strong`, `surface_panel`, `border_hairline`, `accent`,
     `editor_cursor`, `editor_selection`, `positive`, `caution`,
     `critical`, …).
   - `ColorTheme::paper()` is the light default; `ColorTheme::ink()` is
     the dark alternate.
   - `ColorTheme::from_config(limn_service::Theme)` resolves the user's
     persisted choice ( `Light` → `paper`, `Dark` → `ink` ).

3. **New render code in `crates/limn-ui/` MUST source colors from
   `ColorTheme` rather than from hardcoded `rgb(0x...)` literals.**
   Wave 10-C will replace the four existing hardcoded sites:

   - `editor.rs:295` ( `EditorView` bg ) → `ColorTheme::surface_app`
   - `editor.rs:296` ( `EditorView` fg ) → `ColorTheme::editor_text`
   - `lib.rs:51` ( `DocumentView` bg ) → `ColorTheme::surface_app`
   - `lib.rs:52` ( `DocumentView` fg ) → `ColorTheme::text_body`

   The line numbers are approximate ( they will shift in the Wave 10-C
   diff ); the mapping by role is what's decided. `DocumentView` maps
   to `text_body` rather than `editor_text` because it renders read-only
   prose, not an editor. Any new `rgb(0x...)` literal that lands in a
   render path after this ADR is a code-review red flag.

4. **`gpui_component::Theme` continues to exist** for components imported
   from gpui-component (`SettingsView`'s Input / Switch / Button etc.).
   The Limn `ColorTheme` and `gpui_component::Theme` are kept in sync at
   the Settings save boundary (Wave 10-D); they are not unified into a
   single type, because gpui-component owns its own theme contract
   upstream and we do not fork it.

We choose this shape because it gives Limn one documented place that
answers "what color is this?" without forcing a fork of gpui-component
or a bespoke wrapper around every render call.

---

## Consequences

### Positive

- **One documented place answers the color question.** A contributor
  adding a new surface reads `visual-language.md` for intent, picks a
  `ColorTheme` role, and uses `ColorPalette` only if no role fits (in
  which case the missing role should be added to `ColorTheme` rather
  than reaching past it).
- **Drift is detectable.** Any new `rgb(0x...)` literal in
  `crates/limn-ui/` is structurally suspicious and shows up in review.
- **Dark mode becomes a real product feature, not just a config bit.**
  `ColorTheme::ink()` defines the entire surface, including overlay
  alpha, hairline alpha, and selection alpha — values that hardcoded
  literals never captured.
- **The single-accent constraint is enforced structurally.** There is no
  `accent_secondary` field on `ColorTheme`. Approved alternates (sepia,
  graphite) live on `ColorPalette` as raw colors that would require a
  per-product theme override to actually ship — adding a second accent
  is therefore a visible decision, not an accident.

### Negative / Trade-offs

- **Two theme systems coexist** (`limn_ui::ColorTheme` and
  `gpui_component::Theme`). Settings save must update both — handled in
  Wave 10-D. The duplication is intentional: gpui-component's `Theme`
  is the contract its components consume, and we do not own that
  contract. The cost is a doubled write at save time.
- **The 14-step neutral ramp is finer than typical 9-step ramps.**
  Contributors may pick the wrong step. Mitigation:
  `visual-language.md` documents the intent of each step (`n-50` = app
  paper, `n-100` = sunken wells, etc.), and `ColorTheme` shields most
  render sites from raw-step choices.
- **A short window of inconsistency.** Between Wave 10-A and Wave 10-C,
  `editor.rs` and `lib.rs` still render the old hardcoded literals
  while `SettingsView` and `PaletteView` already render against a
  theme. The inconsistency is a known follow-up, not an unknown.
- **Drift risk between `limn_ui::ColorTheme` and
  `gpui_component::Theme`.** If either system's semantic values change
  ( our palette adjustment, or an upstream gpui-component theme
  change ) without the other being updated, components rendered by
  gpui-component ( e.g. `SettingsView`'s `Input`, `Button`, `Switch` )
  will visually diverge from Limn-rendered surfaces ( `EditorView`,
  `DocumentView`, `AppShell` chrome ). Neither system catches this
  automatically. Wave 10-D's implementation MUST include an assertion
  or snapshot test that confirms the values written into
  `gpui_component::Theme` match the corresponding `ColorTheme` fields,
  so the doubled write is at least self-checking.
- **Cognitive load for new contributors.** "Which API do I use to get
  the background color?" now has two answers depending on what's being
  rendered: components ported from gpui-component use `cx.theme()`;
  Limn-owned views use `cx.global::<ColorThemeGlobal>().0` ( landed in
  Wave 10-C; the type is `ColorThemeGlobal(pub ColorTheme)` in
  `crates/limn-ui/src/theme.rs` ). `docs/design/visual-language.md`
  documents this
  split ( see the "Which theme API to call" table in the Editor
  specifics section ), but it is one more rule to learn. Mitigation:
  the table is the single place a new contributor needs to read.

### Neutral

- **No change to `LimnConfig` schema.** `theme = "light" | "dark"`
  remains the user-facing knob; this ADR only changes how the chosen
  value is consumed.
- **`run_read_only` ( the M1 read-only path ) was migrated in Wave
  10-C:** it now registers
  `ColorThemeGlobal(ColorTheme::from_config(config.theme))` at startup,
  so `DocumentView::render` sources its background from
  `ColorTheme::surface_app` and its text color from
  `ColorTheme::text_body` — the same theme global pattern used by the
  editable path. The read-only path has no settings UI, so there is no
  live switch, but the persisted `LimnConfig.theme` is honored at
  launch.
- **`limn-core` and `limn-service` remain gpui-free.** `theme.rs`
  intentionally lives in `limn-ui` because `Rgba` is a gpui type — see
  the relationship-to-other-ADRs section below.

---

## Considered Alternatives

### Alternative A: Use only `gpui_component::Theme` and add Limn-specific roles via a wrapper

- Summary: One theme type. Define a `LimnTheme` wrapper around
  `gpui_component::Theme` that adds Limn-specific roles (e.g.
  `editor_focus_dim`, `editor_syntax`, `editor_selection`) by deriving
  from gpui-component's base.
- Reason for rejection: gpui-component's `Theme` schema is owned
  upstream; bending it for Limn's semantic roles would either fork it
  (and inherit upstream-merge maintenance debt) or add wrapper
  indirection on every render call. The wrapper approach also defers
  the decision rather than resolving it — Wave 10-C still has to write
  *somewhere*. Keeping a Limn-owned `ColorTheme` next to a Limn-owned
  `visual-language.md` is the cheaper structural choice; the
  duplicated-write cost at Settings save is bounded and explicit.

### Alternative B: Hardcode the design tokens as `const fn` rather than `pub fn` methods

- Summary: Treat the palette as compile-time constants
  (`pub const N_50: Rgba = …`) instead of `pub fn n_50() -> Rgba`.
- Reason for rejection: `gpui::rgb()` is not a `const fn` in the
  pinned gpui revision. Working around it would mean either writing raw
  `f32` literals (losing the readability of `rgb(0x00f0_f3f4)`) or
  duplicating gpui's `u8 -> f32 / 255.0` computation by hand. The `pub
  fn` form is clearer at the call site and the performance cost of an
  inlinable accessor is negligible.

### Alternative C: Defer to a future "design-tokens crate" and ship a stub theme now

- Summary: Mark the palette as provisional, ship a stub `ColorTheme`
  with just `paper` / `ink` covering the immediate needs, and design a
  proper design-tokens crate later.
- Reason for rejection: Wave 10-C needs a real palette today to
  replace the hardcoded literals in `editor.rs` and `lib.rs`. Shipping
  a stub means a second migration once the "real" tokens land — the
  same Wave-10-C work, redone. The work is not different enough to be
  worth doing twice.

---

## Relationship to other ADRs

- **[ADR-0002](0002-three-crate-layered-architecture.md) (three-crate
  layered architecture).** Respected. `theme.rs` lives in `limn-ui`
  because `Rgba` is a gpui type. `limn-core` and `limn-service` remain
  gpui-free; `ColorTheme::from_config(limn_service::Theme)` is a one-way
  dependency from `limn-ui` into `limn-service`'s enum, not the reverse.
- **[ADR-0007](0007-user-configuration-via-toml-file.md) (user
  configuration via TOML).** Respected. `LimnConfig.theme` remains the
  only persisted theme choice; `ColorTheme::from_config()` consumes it.
- **[ADR-0010](0010-settings-view-and-view-switching.md) (settings view
  + theme reactivity).** Completed by this ADR plus Wave 10-C/10-D.
  ADR-0010's addendum noted "render pixel-verification remains outside
  gpui headless test coverage"; that gap persists at the pixel layer,
  but the value-side of the contract (`ColorTheme` fields, `from_config`
  mapping, `paper != ink`) is exhaustively unit-tested in
  `crates/limn-ui/src/theme.rs`.

---

## Links

- Narrative: [`docs/design/visual-language.md`](../design/visual-language.md)
- Rust implementation: [`crates/limn-ui/src/theme.rs`](../../crates/limn-ui/src/theme.rs)
- Related ADRs: [ADR-0002](0002-three-crate-layered-architecture.md),
  [ADR-0007](0007-user-configuration-via-toml-file.md),
  [ADR-0010](0010-settings-view-and-view-switching.md)
- Reference: `AGENTS.md` (keyboard-first, English-only docs),
  `ARCHITECTURE.md` (crate boundaries)
