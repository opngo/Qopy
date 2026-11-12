// Module-level state shared across page instances. Pages remount on every
// route change, but this module does not - so a lock held here works even
// if two stale component instances somehow receive the same key event.

const PASTE_LOCK_MS = 1500;
let pasteLockedUntil = 0;

/**
 * Returns true and arms a short lock when a paste may start; returns false
 * while a previous paste is still in flight (prevents double pastes).
 */
export const beginPaste = (): boolean => {
  const now = Date.now();
  if (now < pasteLockedUntil) return false;
  pasteLockedUntil = now + PASTE_LOCK_MS;
  return true;
};
