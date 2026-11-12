import { applyTheme, getStoredTheme, isTheme } from "~/lib/theme";

export default defineNuxtPlugin(() => {
  const { $settings } = useNuxtApp();

  // Apply the cached theme synchronously so the first paint uses it.
  applyTheme(getStoredTheme());

  // Reconcile with the value persisted by the app (survives browser reset).
  $settings
    .getSetting("theme")
    .then((stored) => {
      if (stored && isTheme(stored)) applyTheme(stored);
    })
    .catch(() => {
      // Tauri backend not ready yet (e.g. plain web preview) - keep cache.
    });
});
