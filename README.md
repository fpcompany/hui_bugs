# `bevy_hui` 0.7.0 / Bevy 0.19 repros

This is a stock-crates.io reproduction: `Cargo.toml` depends on `bevy = "0.19"`
and `bevy_hui = "0.7.0"`; there are no patches or vendored dependencies.

Run either independently:

```sh
cargo run --bin bug1_relayout
cargo run --bin bug2_root_scope
```

## Bug 1 — static templates are dirtied every frame

`bug1_relayout` instantiates `assets/bug1_rows.html` 40 times. Each instance
has 20 static text rows (800 text nodes, plus 40 container nodes). Nothing
animates and no interaction is needed.

Once per second, the program logs total and `Changed` counts for `Node`,
`TextFont`, `HoverTimer`, and `PressedTimer`. On a static UI, the changed counts
should settle at zero after startup. Instead, they remain at the complete
template population on every frame (the per-second observation samples the
current frame): about 840 `Node`s, 800 `TextFont`s, and 840 of each public timer.
`FrameTimeDiagnosticsPlugin` and `LogDiagnosticsPlugin` also print FPS/frame
time. Press `SPACE` to despawn all hui roots, or `H` to set their roots to
`Visibility::Hidden`; despawning removes the work, while hiding leaves the
static style passes and associated cost in place.

Expected: static, non-interactive UI stops changing and does not continually
force UI layout/text work.

Observed: every `HtmlStyle` is mutably processed every update. In particular,
`apply_computed` writes through `Mut` to `Node`, `TextFont`, `TextColor`,
`BackgroundColor`, and `BorderColor`, so Bevy sees them as changed even where
their values are identical. This drives `ui_layout_system`'s taffy sync and
text's rerender detection continuously. `HoverTimer` and `PressedTimer` are
also public and counted here; their timer system obtains mutable references and
calls `forward`/`backward` even after the timer is saturated at its endpoint.

Responsible upstream locations in `bevy_hui` v0.7.0:

- [`src/styles.rs:394-402`](https://github.com/Lommix/bevy_hui/blob/v0.7.0/src/styles.rs#L394-L402): `update_node_style` iterates every `HtmlStyle` and unconditionally calls `apply_computed`.
- [`src/styles.rs:143-235`](https://github.com/Lommix/bevy_hui/blob/v0.7.0/src/styles.rs#L143-L235): `apply_computed` takes mutable UI components and assigns the computed values.
- [`src/styles.rs:69-123`](https://github.com/Lommix/bevy_hui/blob/v0.7.0/src/styles.rs#L69-L123): `continues_interaction_checking` obtains/mutates the interaction timers without checking whether an operation can still change them.

The downstream fix shape is to gate per-entity style work on `Changed<HtmlStyle>`
or an interaction timer that actually changed, or `UiActive` being added/removed,
and to skip timer operations at zero/max. There is one important trap:
`apply_computed` memoizes a font path as a handle inside `HtmlStyle`; that write
must use `bypass_change_detection`, otherwise it self-retriggers the gate.

## Bug 2 — root-node property expressions never compile

`bug2_root_scope` loads two small templates, deliberately separated so the
results cannot obscure one another. `assets/bug2_root_scope.html` has a root
`<node>` and direct child `<node>` that both use `background="{bg}"`; their
white/gold borders make the two same-colour regions distinguishable.
`assets/bug2_root_text.html` is an independent root `<text>` containing
`{label}`. The spawn supplies a bright red `bg` and an obvious label.

Expected: the upper outlined root and child panels are both red, and the lower
dark panel says `ROOT TEXT SHOULD BE COMPILED`.

Observed: only the upper child panel is red; its root remains transparent/default.
The lower panel retains literal `{label}`. The result is visible in the window
and is also printed once after construction by reading all three
`BackgroundColor`s and the root `Text` component.

Responsible upstream locations in `bevy_hui` v0.7.0:

- [`src/build.rs:461-463`](https://github.com/Lommix/bevy_hui/blob/v0.7.0/src/build.rs#L461-L463): `TemplateScope` is inserted only if `entity != self.scope`, so the template root lacks it.
- [`src/compile.rs:68-89`](https://github.com/Lommix/bevy_hui/blob/v0.7.0/src/compile.rs#L68-L89): `compile_node` requires `&TemplateScope`, so it returns before compiling a root's attributes.
- [`src/compile.rs:34-54`](https://github.com/Lommix/bevy_hui/blob/v0.7.0/src/compile.rs#L34-L54): `compile_text` likewise requires `&TemplateScope`, so root text content is skipped.

The one-line fix in each compile system is to accept an optional scope and use
the node itself as the fallback scope:

```rust
scope.map(|s| **s).unwrap_or(entity)
```
