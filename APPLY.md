# Qopy changes - how to apply to your fork

These are the ONLY 39 files changed for the paste-fix + features update
(two rounds of fixes combined). Copy this folder's contents over the root
of your qopy fork (overwrite same-named files, add the new ones), then:

    bun install        # installs highlight.js (package.json + bun.lock included)
    bun dev            # or: bun build

## What's in it

Round 1 - paste/selection fixes + features:
- Selection tracked by item id (no more reset/shift when the list rebuilds)
- Clipboard-event dedup (SHA-256 fingerprinting) in Rust
- Paste hardening: supports code+file types, longer pre-paste delay,
  hotkey ignored during a programmatic paste
- Code highlighting (highlight.js, ~20 languages) + Rust code detector
- Themes (Ember/Midnight/Forest/Crimson/Paper) + settings gear icon
- Pin entries, per-entry actions menu (copy/pin/delete), Delete key,
  clear-history, empty states, toasts

Round 2 - bug fixes from testing:
- lib/keyboard.ts + lib/appState.ts (NEW): single shared keyboard
  instance + paste lock. Fixes "down arrow skips 2" and "double paste"
  (root cause: per-page keyboard instances + never-unlistened Tauri
  focus listeners stacked multiple key handlers across page remounts).
- Down arrow now works in search.
- Image/file thumbnails: reactive storage (was shallowRef, so every row
  showed the first painted image).
- File entries pointing at images (SVG/PNG/JPG/GIF/WebP/BMP/AVIF/ICO)
  get real preview thumbnails in the list and the full preview pane.
- Gear + pin icons: replaced with canonical Material Design paths.
- Syntax highlighting expanded to 40 languages + aliases, with
  highlightAuto fallback for unknown language labels.
- Settings page: theme swatches in a row, compact spacing that fits
  the 474px window.

## File list (26 modified, 13 added) - all source, no binaries

ADDED:
  components/ActionsMenu.vue
  components/Icons/Copy.vue
  components/Icons/Delete.vue
  components/Icons/Pin.vue
  components/Icons/Settings.vue
  lib/appState.ts
  lib/keyboard.ts
  lib/theme.ts
  plugins/theme.ts
  src-tauri/src/db/migrations/v4.sql
  src-tauri/src/utils/code.rs

MODIFIED:
  README.md, app.vue, bun.lock, package.json
  components/BottomBar.vue, components/Result.vue, components/TopBar.vue
  components/Icons/{Cmd,Code,Enter,File,Image,K,Link,Text}.vue
  lib/selectedResult.ts
  pages/index.vue, pages/settings.vue
  plugins/history.ts
  src-tauri/src/api/{clipboard,hotkeys}.rs
  src-tauri/src/db/history.rs
  src-tauri/src/main.rs
  src-tauri/src/utils/{mod,types}.rs
  styles/{index,settings}.scss
  types/types.ts

Note: src-tauri/Cargo.toml is NOT changed - no new Rust dependencies were
added (the code detector is dependency-free), so `cargo build` picks up
src-tauri/src/utils/code.rs automatically via utils/mod.rs.
