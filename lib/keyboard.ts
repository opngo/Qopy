import { useKeyboard } from "@waradu/keyboard";

// A single keyboard manager shared by every page.
//
// Previously each page created its own useKeyboard() instance and re-armed it
// on every window focus. Because page instances remount on every route change
// (and Tauri event listeners were never removed), multiple window keydown
// handlers accumulated - which is what made the down arrow jump two rows and
// Enter paste twice.
//
// One shared instance can never stack window listeners. Pages coordinate with
// armKeyboard(): every onMounted wipes ALL registered key handlers and
// re-attaches exactly one window listener, then registers only its own
// handlers. Pages deliberately do NOT stop/clear in onUnmounted - the page
// being navigated to always clears first thing in its onMounted, before the
// user can press anything, so nothing leaks across routes either way.
export const appKeyboard = useKeyboard();

/** Wipe all key handlers and ensure exactly one window listener is armed. */
export const armKeyboard = (): void => {
  appKeyboard.clear();
  appKeyboard.init();
};
