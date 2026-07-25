# Cursor Visual Rework — Plan

**Goal:** Make the launcher and the editor look and behave like Cursor's Ctrl+K panel — top-anchored rounded panel, drop shadow, grouped icon rows, highlighted match characters, content-sized height, footer hint bar — on Cursor's near-black greys.

Measured against a screenshot of the real panel. Where this plan and that screenshot disagreed, the screenshot won, and the disagreements are called out inline so the reasoning stays visible.

Steps use checkbox (`- [x]`) syntax so progress is trackable. Nothing here depends on a plugin or a particular runner.

**Decisions taken (2026-07-25):**

1. Retarget the default palette to Cursor's greys, sampled from a screenshot of the real app.
2. Rework the launcher and the editor in one pass, on shared widget primitives.
3. Transparent window with rounded corners and a shadow, unconditionally. No square fallback, no config flag.
4. **Group headers appear only when the query is empty.** Typing gives a flat, score-sorted list. Grouping and relevance want different row orders, and relevance wins the moment a query exists.
5. **Fuzzy matching stays on `name + detail`; match positions are split at the name boundary** so both spans highlight. Searching by path or host keeps working.
6. **The window resizes only when the clamped row count changes**, not on every keystroke.
7. **The editor is a fixed 600×560**, resized on `Ctrl+E` toggle, sharing the launcher's top edge and its one window.
8. **Bundle Inter Regular + SemiBold.** Determinism is worth the 300-400 KB.
9. **No animation anywhere.** Cursor's panel does not animate; neither will ours.
10. **No config migration.** Single-user tool — delete `config.toml` and re-add entries.

**Architecture:** `app.rs` is 912 lines and holds state, update, both views, and every inline style closure. Split it: `app.rs` keeps state and `update`; `theme.rs` grows into palette plus layout metrics; a new `ui/` module holds the two views and the handful of builders they share. Extract only what both views call — nothing speculative.

**Tech stack:** iced 0.14 (verified present: `widget::rich_text`/`span`, `widget::mouse_area::on_enter`, `operation::snap_to`/`scroll_to`/`focus_next`, `window::resize`/`move_to`, `window::Position::SpecificWith`, `transparent` in `window::Settings`, `shadow` on `container::Style` and `button::Style`), nucleo-matcher 0.3 (`Atom::indices`). `animation::Animation` was verified present but decision 9 retires it.

---

## Why the interaction work is the bulk of it

Cursor's palette looks the way it does partly because of how it behaves. Four of the visual traits are behaviour, not paint:

- **Content-sized panel.** Cursor's palette is as tall as its results. Ours is a fixed 500×400 (`src/app.rs:833`), so two entries sit in a mostly empty box.
- **The selected row is always on screen.** Ours isn't. The `scrollable` at `src/app.rs:775` has no `Id`, so nothing scrolls it. Arrow past the visible rows and `selected_index` keeps moving invisibly. With more than about eight entries the launcher is broken by keyboard.
- **Hover and keyboard agree.** Ours disagree: hovering row 5 tints it (`src/app.rs:743`) while row 1 stays selected, so two rows look active at once.
- **Matched characters are lit.** `fuzzy.rs` throws away match positions — it returns only indices (`src/fuzzy.rs:41`).

Three more defects surfaced while reading, all worth fixing in the same pass:

- **The key handler is view-blind.** `src/app.rs:804-825` maps events with no knowledge of `current_view`. Arrow keys move the launcher selection while you are typing in the editor, and Enter saves the editor entry no matter which field has focus — press Enter after typing a port and it saves.
- **The editor has no focus order.** `operation::focus_next` is unused; Tab does nothing between the fields.
- **Delete confirm is a trap.** `EditorDelete` swaps the button in place (`src/app.rs:270`). Escape does not cancel the pending confirm — it cancels the whole editor.

Two things I noticed and am leaving alone, per "mention, don't touch": `Settings.accent` (`src/config.rs:23`) is parsed and never read, and `rebuild_filtered_list` re-`format!`s the whole entry list on every keystroke (`src/app.rs:391`). Task 3 puts `accent` to work as the match-highlight colour, which retires the orphan honestly. The allocation is invisible at this scale — leave it.

---

## Palette

Read off a screenshot of the actual Cursor build in use, not from VS Code's published theme tokens. **These are eyeballed to within a few points — sample them with a colour picker before locking them in.**

| Token        | Old (Claude) | New (Cursor) | Used for                        |
| ------------ | ------------ | ------------ | ------------------------------- |
| `background` | `#1F1E1C`    | `#141414`    | editor field wells              |
| `surface`    | `#262624`    | `#212121`    | the panel itself                |
| `foreground` | `#FAF9F5`    | `#D4D4D4`    | entry names, input text         |
| `muted`      | `#9C9A92`    | `#858585`    | detail text, placeholder, hints |
| `border`     | `#30302E`    | `#333333`    | panel border, hairlines         |
| `highlight`  | `#D97757`    | `#2F2F2F`    | selected-row fill               |
| `accent`     | `#E08B6F`    | `#D9A05B`    | matched characters              |
| `danger`     | `#EE8884`    | `#F14C4C`    | delete                          |

**The palette is monochrome.** Nothing in Cursor's Ctrl+K panel is coloured — not the selected row, not the filter tabs, not the section headers, not the footer. The only place `accent` appears in our design is matched characters, and even that is unverified: no query was typed in the reference screenshot, so I have not seen Cursor highlight a match. `#D9A05B` is carried over from the amber in Cursor's editor chrome on the assumption it is the same accent. If it looks wrong against the greys, that is the one value to change, and changing it affects nothing else.

Three corrections against the VS Code Dark Modern values this plan first carried:

**The accent is warm, not blue.** Cursor's active tab title and its decorative chrome are amber. `#2AAAFF` would have been VS Code's accent, not this app's. The old Claude terracotta `#D97757` was closer to right than the blue was — the accent barely moves, and mostly cools.

**The selected row is a neutral lift, not an accent fill.** Cursor marks selection with a lighter grey at radius ~6 plus a faint outline, not a coloured block. This is the biggest single change, and it simplifies the code: every row in both views currently inverts text colour when selected (`if is_selected { bg } else { fg }` at `src/app.rs:604`, `:729`, `:732`, `:750`). Against a neutral lift there is nothing to invert. All four sites collapse to a plain `fg`.

**Borders are far subtler.** `#333333` on `#212121` is a hairline you have to look for. VS Code's `#454545` would draw boxes around everything and read as a different app.

`highlight` also changes meaning: today it is both the selection fill and every focus accent. Those split — `highlight` becomes the neutral selection lift, `accent` takes matched characters and focus rings.

**Existing installs keep their old colours.** serde reads whatever is already in `config.toml`, and `save_to` (`src/config.rs:148`) writes every `Settings` field back explicitly, so any editor save pins the current palette to disk. Only fresh installs get greys.

Per decision 10 there is no migration: delete `%APPDATA%\terminal-switcher\config.toml` and re-add entries. Single-user tool, and a migration cannot tell "still on old defaults" from "deliberately chose terracotta" anyway. Delete the file before testing or you will review this whole rework against the Claude palette.

## Metrics

One `font_size` key drives four sizes, so the config stays a single knob:

| Element      | Size            | Notes                          |
| ------------ | --------------- | ------------------------------ |
| input text   | `font_size + 1` | 15 at the default 14           |
| entry name   | `font_size`     | 14                             |
| detail text  | `font_size - 1` | 13, muted                      |
| hint bar     | `font_size - 3` | 11, muted                      |

| Metric             | Value  |
| ------------------ | ------ |
| panel width        | 600    |
| top edge           | 30% of screen height |
| shadow inset       | 16     |
| panel radius       | 12     |
| panel side padding | 8      |
| input row height   | 44     |
| filter row height  | 30     |
| section header     | 24     |
| entry row height   | 30     |
| row radius         | 6      |
| row inset          | 8      |
| hint bar height    | 24     |
| max visible rows   | 9      |

Measured off the reference screenshot: panel about 597 wide, rows on a 29px pitch, selected row inset 8 from each panel edge, footer about 22 tall.

Cursor rounds more than VS Code does — list rows about radius 6, the panel about 12. Rows are roomier than a VS Code quick-pick row. Erring generous is closer to the target than erring tight.

**Anchor the top edge, do not centre the panel.** In the reference the panel's top sits near 30% of screen height. I cannot tell from one screenshot whether Cursor pins that top edge or re-centres as results change, but pinning it is the right call regardless: the panel resizes as results narrow (Task 2), and a centred panel would jump under the cursor as you type. Only the bottom edge should move — including when the editor opens at its taller fixed size.

---

### Task 1: Palette and metrics in `theme.rs`

**Files:** Modify `src/theme.rs`, `src/config.rs`

- [x] Replace the eight `default_*` colour fns in `src/config.rs:35-42` with the Cursor values from the table.
- [x] Update the `AppColors::from_settings` fallbacks in `src/theme.rs:39-52` to match. They are duplicated today — have them read the `default_*` fns instead of repeating literals.
- [x] Add `accent: Color` to `AppColors` (`src/theme.rs:25`). It is the one field `Settings` has and `AppColors` drops.
- [x] Add a `Metrics` struct holding the numbers from the metrics table, with the four derived font sizes computed from `settings.font_size`.
- [x] Fix the tests that assert on the old hex values: `src/config.rs:239-249`, `src/config.rs:263-266`, `src/theme.rs:121-124`.

**Verify:** `cargo test` passes.

---

### Task 2: Content-sized, top-anchored, transparent window

**Files:** Modify `src/app.rs:830-851`, `src/main.rs:116-124`

- [x] Add `fn panel_height(visible_rows: usize, headers: usize) -> f32` to `theme.rs`, a pure function: `input + rows.min(9) * row_height + headers * header_height + hint_bar + padding + 2 * shadow_inset`. Unit-test it at 0, 1, 9, and 50 rows, with 0 and 2 headers. Per decision 4, `headers` is 2 when the query is empty and 0 otherwise — there is no one-header case.
- [x] Add `fn editor_height() -> f32` returning 560. The editor is a form; it has no reason to breathe with content.
- [x] In `window_settings`, set `transparent: true`, `size` to 600 × `panel_height(entry_count, 2)`, and `position: Position::SpecificWith(|win, monitor| Point::new((monitor.width - win.width) / 2.0, monitor.height * 0.30))`.
- [x] Set `resizable: true` and clamp with `min_size`/`max_size`. `window::resize` is unreliable against `resizable: false` on some backends; if resize works with it false, revert this.
- [x] Call `.transparent(true)` on the application builder in `main.rs`.
- [x] Track the last-issued height on `App`. Have `rebuild_filtered_list` compute the new height and issue `window::resize` **only when it differs** (decision 6), then thread the `Task` through its three callers (`src/app.rs:117`, `:265`, `:312`). Because height is a function of `min(matches, 9)`, narrowing 50→30→12 matches issues no resize at all; only crossing below 9 does.
- [x] Have `Message::ToggleEditor` (`src/app.rs:178`) resize to `editor_height()` on the way in and back to `panel_height(..)` on the way out (decision 7). The top edge is fixed, so the window grows downward.

**Verify:** `cargo test` for `panel_height`. Manually: launcher opens near the top of the screen; typing a query that narrows results shrinks the panel but the top edge stays put; typing within the 9-row clamp produces no resize at all; Ctrl+E grows the window downward to the editor size and back.

---

### Task 3: Match highlighting

**Files:** Modify `src/fuzzy.rs`, `src/app.rs`

- [x] Change `FuzzyMatcher::filter` to return `Vec<(usize, Vec<u32>)>` — entry index plus matched character positions — using `Atom::indices` instead of `Atom::score`. Keep the score sort.
- [x] Test it: `"prd"` against `"Prod Server"` returns positions for P, r, d.
- [x] Store the positions on `App` alongside `filtered_indices`.
- [x] Split each position list at the name boundary (decision 5) and render both the name and the detail with `rich_text`, one `span` per run, matched runs in `colors.accent` with the semibold face from Task 4.

Matching stays on the combined `format!("{} {}", name, detail)` haystack built at `src/app.rs:395`, so searching by path or host keeps working. **The boundary is `name.chars().count()`, not `name.len()`** — nucleo returns char indices into a `Utf32Str`, and any non-ASCII character in a name would desynchronise a byte-based split. Positions below the boundary belong to the name; positions above `boundary + 1` belong to the detail, offset by `boundary + 1`; the position at the boundary itself is the joining space and is dropped.

- [x] Test the split directly: an entry named `Ünicode` with a matching path proves the char-vs-byte boundary.

**Verify:** `cargo test` for the index test. Manually: typing `prd` lights P, r, and d in amber.

---

### Task 4: Fonts and icons

**Files:** Modify `src/main.rs`, add `assets/`

- [x] Bundle Inter Regular and Inter SemiBold, load both with `.font(include_bytes!(..))`, set Regular as `.default_font`. SemiBold is what Task 3 needs for matched runs.
- [x] Bundle a two-glyph Codicons subset (`folder`, `remote`) and add `Entry::icon()` returning the codepoint.
- [x] Note the binary size delta in the commit message. Inter plus a subset is roughly 300-400 KB against a stripped LTO build.

Codicons is MIT and is VS Code's own icon font, so it is the on-brand choice. Subset it — do not ship the whole face for two glyphs.

**Verify:** `cargo build --release`. Text renders in Inter, both entry types show their icon.

---

### Task 5: Extract shared UI primitives

**Files:** Add `src/ui/mod.rs`, `src/ui/launcher.rs`, `src/ui/editor.rs`; shrink `src/app.rs`

- [x] Move `launcher_view` (`src/app.rs:686-793`) and `editor_view` (`src/app.rs:408-684`) into `ui/`.
- [x] Pull out only the builders both views need: `panel()` (transparent outer margin, rounded shadowed container), `field()` (the text-input style duplicated at `src/app.rs:417` and `:699`), `row_item()` (icon, rich-text label, muted detail, selection state), `hint_bar()`.
- [x] Everything single-use stays inline. The radio styles, the danger button, the first-run banner are editor-only — leave them there.

`app.rs` should land near 400 lines, holding state and `update`.

**Verify:** `cargo build` and `cargo test` pass. No behaviour change yet — this task is a move.

---

### Task 6: View-scoped keyboard handling

**Files:** Modify `src/app.rs:795-828`, `src/app.rs:48-75`

- [x] Replace the per-key messages from `listen_with` with one `Message::KeyPressed { key, modifiers }`. The closure cannot see `self`, which is why the current handler is view-blind.
- [x] Branch on `current_view` inside `update`:
  - **Launcher:** Up/Down move, Enter launches, Escape hides, Ctrl+E opens the editor, Home/End jump to first/last, Ctrl+N/Ctrl+P move (Cursor supports these).
  - **Editor:** Tab/Shift+Tab call `operation::focus_next`/`focus_previous`, Enter saves only when the form is valid, Escape cancels a pending delete confirm first and closes the editor only on a second press.
- [x] Drop `MoveUp`, `MoveDown`, `KeyEnter`, `KeyEscape` from `Message` once nothing sends them.

**Verify:** Manually — typing a port in the editor and pressing Enter saves; arrow keys in the editor no longer move the hidden launcher selection; Escape on a pending delete cancels the confirm, not the editor.

---

### Task 7: Selection follows scroll and hover

**Files:** Modify `src/app.rs`, `src/ui/launcher.rs`

- [x] Give the launcher `scrollable` an `Id`.
- [x] On every selection move, return `operation::snap_to(id, RelativeOffset { x: 0.0, y: selected as f32 / (len - 1) as f32 })`, guarding `len <= 1`.
- [x] Wrap each row in `mouse_area(..).on_enter(Message::HoverAt(i))`, setting `selected_index`. Hover and keyboard then share one selection and only one row ever looks active.
- [x] Keep click-to-launch. With hover driving selection, a click always launches the row that looks selected, so no extra step is needed.

**Verify:** Manually with 30+ entries — holding Down scrolls the list and the selected row stays visible; moving the mouse over a row selects it and deselects the old one.

---

### Task 8: Row and panel styling

**Files:** Modify `src/ui/*`

- [x] Panel: 16px transparent margin, then a container at radius 12, `surface` fill, 1px `border`, `Shadow { color: rgba(0,0,0,0.5), offset: (0, 8), blur_radius: 32 }`.
- [x] Input row: flush, no box, no icon, 44px tall, ~16px left padding, placeholder in `muted`. Cursor has no search glyph — do not add one. That also drops a glyph from the Task 4 font subset.
- [x] Section headers: `Directories` and `SSH Hosts` at `font_size - 2` in `muted`, left-aligned at the row inset, 24px tall, title case. Not uppercase. **Rendered only when `search_query` is empty** (decision 4) — as soon as a query exists the list goes flat and score-sorted, so the best match is always row 1.
- [x] Headers are not selectable. `selected_index` indexes entries, not rendered rows, so arrow keys skip headers for free — but the `snap_to` ratio in Task 7 must account for the header rows above the selection, or the scroll target drifts.
- [x] Entry rows: 30px, 8px inset, radius 6 — icon, then rich-text name, then the muted detail inline immediately after it.
- [x] Selected row: `highlight` fill, 1px faint outline, text stays `foreground`. Matched characters stay `accent`.
- [x] Delete the selected-state colour inversion at `src/app.rs:604`, `:729`, `:732`, and `:750`. A neutral lift needs no inverted text, and leaving the inversion in place would make selected rows unreadable.
- [x] Style the scrollbar as an overlay slab — no track, muted thumb.
- [x] Hint bar: hairline above, `⇅ Select    ↵ Open    ^E Edit    esc Close` at `font_size - 3` in `muted`, left-aligned.
- [x] Apply the same panel, field, and button treatment to the editor.

Two corrections from the reference screenshot, both against what this plan said earlier:

**Detail text goes inline, not right-aligned.** Cursor uses both patterns — right-aligned metadata for agent rows, inline dimmed text straight after the name for file rows. Our entries are paths and hosts, so the file pattern is the closer match: `My Project  ~/projects/myapp`. That also lets the right-aligned kind label go, since the icon already says which type it is. One less thing to lay out.

**The footer is not a deviation.** Cursor's panel has one: `⇅ Select`, `↵ Open`, and a filter hint, left-aligned and muted under a hairline. Earlier this plan called the hint bar an invention that needed justifying. It does not — copy it. Note the style is key-glyph then verb with wide gaps, not middot-separated lowercase.

**Verify:** Manually against a Cursor screenshot.

---

### ~~Task 9: Motion~~ — cut

**Cut by decision 9.** Cursor's panel does not animate, and neither do Alfred, Raycast, or VS Code quick-pick. A fade-in would add perceived latency to the one interaction the hotkey exists for, and a sliding selection lags visibly under arrow-key repeat.

Cutting it also removes the `window::frames()` redraw pump, which matters for a process that sits in the tray all day. Panel appears at 0ms; selection snaps.

---

### Task 9: Documentation

**Files:** Modify `README.md`

- [x] Update the config sample at `README.md` (the `[settings]` block) with the new hex values and drop the "Claude design language" comment.
- [x] Add Home/End and Ctrl+N/Ctrl+P to the keybindings table.
- [x] Note that existing configs keep their old colours, and that picking up the new defaults means deleting `config.toml` and re-adding entries.

**Verify:** The README sample parses — paste it into `Config::load_from` in a test if you want the guarantee.

---

## Risks

**Transparency is the real one.** You chose no fallback, so these have to be handled rather than switched off:

- **Windows:** `transparent: true` alongside `skip_taskbar` (`src/app.rs:842`) is the untested combination. If the panel shows a black rectangle instead of transparency, the window needs `WS_EX_LAYERED` and DWM blur-behind — reachable through `window::raw_id` and a `windows` crate call, but that is a day of work, not an hour.
- **Linux:** `override_redirect` (`src/app.rs:847`) bypasses the window manager. Transparency then needs a running compositor. Without one the corners render black. There is no way to detect this cleanly at runtime.
- **Shadow clipping:** iced draws the shadow inside its own surface. The 16px inset in Task 2 is what stops it being clipped at the window edge. If the shadow looks cut off, the inset is too small for the blur radius.

**Smaller:**

- `window::resize` against `resizable: false` — Task 2 sets it true as a precaution; confirm which way it actually behaves and simplify.
- Match-position splitting uses char indices, not byte indices. Get this wrong and highlights land on the wrong characters the first time a name contains non-ASCII.
- The `snap_to` ratio in Task 7 must account for header rows in browse mode, or the scroll target drifts.
- Inter plus Codicons adds roughly 300-400 KB to a binary that is currently stripped and LTO'd for size.

### ~~Task 11: Filter tabs~~ — cut

**Cut before implementation.** Proposed after the reference screenshot showed Cursor's `All · Agents · Files · Actions · Settings` segmented filter, then withdrawn once decision 4 landed. Cursor's filter spans five categories over a large corpus; ours would span two, and the empty-query browse view already separates them under headers. That is a segmented control, a `filter` field on `App`, a Ctrl+Tab binding, and header-suppression logic — to hide one of two groups you can already see.

Revisit if entries ever gain a third type.

---

## Order

Nine tasks, two cut. Tasks 1-4 are independent and can go in any order. Task 5 should land before 6-8 or those tasks edit code that is about to move. Task 9 (documentation) is last.

## What is not in scope

Recent-entries ordering, keybinding chips on rows, a settings pane, multi-monitor placement, filter tabs (cut above), and animation (cut above). Say so if you want any of them and they become their own plan.

Group headers were listed here until the reference screenshot showed them carrying most of the panel's structure. They are now part of Task 8 — but only in browse mode, per decision 4.
