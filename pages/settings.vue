<template>
  <main>
    <div class="top-bar">
      <NuxtLink to="/" class="back">
        <img src="../public/back_arrow.svg" />
        <p>Back</p>
      </NuxtLink>
    </div>
    <div class="settings-container">
      <div class="settings">
        <div class="section">
          <div class="section-title">Appearance</div>
          <div class="row">
            <div class="names">
              <p>Theme</p>
            </div>
            <div class="actions theme-actions">
              <button
                v-for="theme in THEMES"
                :key="theme.id"
                class="theme-swatch"
                :class="{ active: currentTheme === theme.id }"
                :title="theme.name"
                @click="selectTheme(theme.id)">
                <span class="swatch-bg" :style="{ background: theme.background }">
                  <span class="swatch-accent" :style="{ background: theme.accent }" />
                </span>
                <span class="swatch-name">{{ theme.name }}</span>
              </button>
            </div>
          </div>
        </div>

        <div class="section">
          <div class="section-title">General</div>
          <div class="row">
            <div class="names">
              <p>Startup</p>
              <p>Qopy Hotkey</p>
              <p>Type filters</p>
            </div>
            <div class="actions">
              <div class="launch">
                <input
                  type="checkbox"
                  id="launch"
                  v-model="autostart"
                  @change="toggleAutostart" />
                <label for="launch" class="checkmark">
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 14 14"
                    fill="none"
                    xmlns="http://www.w3.org/2000/svg">
                    <g>
                      <rect width="14" height="14" />
                      <path
                        id="Path"
                        d="M0 2.00696L2.25015 4.25L6 0"
                        fill="none"
                        stroke-width="1.5"
                        stroke="var(--icon, #E5DFD5)"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        transform="translate(4 5)" />
                    </g>
                  </svg>
                </label>
                <p for="launch">Launch Qopy at login</p>
              </div>
              <div class="launch">
                <input
                  type="checkbox"
                  id="typeFilters"
                  v-model="typeFiltersOn"
                  @change="toggleTypeFilters" />
                <label for="typeFilters" class="checkmark">
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 14 14"
                    fill="none"
                    xmlns="http://www.w3.org/2000/svg">
                    <g>
                      <rect width="14" height="14" />
                      <path
                        id="Path"
                        d="M0 2.00696L2.25015 4.25L6 0"
                        fill="none"
                        stroke-width="1.5"
                        stroke="var(--icon, #E5DFD5)"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        transform="translate(4 5)" />
                    </g>
                  </svg>
                </label>
                <p>Filter chips below search</p>
              </div>
              <div
                @blur="onBlur"
                @focus="onFocus"
                class="keybind-input"
                ref="keybindInput"
                tabindex="0"
                :class="{ 'empty-keybind': showEmptyKeybindError }">
                <span class="key" v-if="keybind.length === 0">Click here</span>
                <template v-else>
                  <span
                    :key="index"
                    class="key"
                    :class="{ modifier: isModifier(key) }"
                    v-for="(key, index) in keybind">
                    {{ keyToLabel(key) }}
                  </span>
                </template>
              </div>
            </div>
          </div>
        </div>

        <div class="section">
          <div class="section-title">History</div>
          <div class="row">
            <div class="names">
              <p>Saved items</p>
            </div>
            <div class="actions">
              <span class="history-count"
                >{{ historyCount }} saved</span
              >
            </div>
          </div>
          <div class="row">
            <div class="names">
              <p>Keep up to</p>
            </div>
            <div class="actions">
              <div class="limit-control">
                <input
                  type="range"
                  min="100"
                  max="5000"
                  step="100"
                  v-model.number="historyLimit"
                  @change="saveHistoryLimit" />
                <span class="limit-value">{{ historyLimit }} items</span>
              </div>
            </div>
          </div>
          <div class="row">
            <div class="names">
              <p>Clipboard Data</p>
            </div>
            <div class="actions">
              <button
                class="danger-btn"
                :class="{ arming: clearArmed }"
                @click="onClearHistory">
                {{ clearArmed ? "Click again to confirm" : "Clear all history" }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
    <BottomBar
      :primary-action="{
        text: 'Save',
        icon: IconsEnter,
        onClick: saveKeybind,
        showModifier: true,
      }" />
  </main>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, reactive, ref } from "vue";
import { platform } from "@tauri-apps/plugin-os";
import { useRouter } from "vue-router";
import { KeyValues, KeyLabels } from "../types/keys";
import { disable, enable } from "@tauri-apps/plugin-autostart";
import BottomBar from "../components/BottomBar.vue";
import IconsEnter from "~/components/Icons/Enter.vue";
import { Key } from "@waradu/keyboard";
import { appKeyboard, armKeyboard } from "~/lib/keyboard";
import { THEMES, applyTheme, getStoredTheme, isTheme } from "~/lib/theme";

const activeModifiers = reactive<Set<KeyValues>>(new Set());
const isKeybindInputFocused = ref(false);
const keybind = ref<KeyValues[]>([]);
const keybindInput = ref<HTMLElement | null>(null);
const lastBlurTime = ref(0);
const os = ref("");
const router = useRouter();
const showEmptyKeybindError = ref(false);
const autostart = ref(false);
const typeFiltersOn = ref(true);
const currentTheme = ref(getStoredTheme());
const clearArmed = ref(false);
let clearTimer: ReturnType<typeof setTimeout> | null = null;
const historyCount = ref(0);
const historyLimit = ref(1000);
const { $settings } = useNuxtApp();
const { $history } = useNuxtApp();
// Shared instance - the main page uses it too, so each page's onMounted
// clears and re-registers exactly one set of handlers (no stacking).
const keyboard = appKeyboard;

const modifierKeySet = new Set([
  KeyValues.AltLeft,
  KeyValues.AltRight,
  KeyValues.ControlLeft,
  KeyValues.ControlRight,
  KeyValues.MetaLeft,
  KeyValues.MetaRight,
  KeyValues.ShiftLeft,
  KeyValues.ShiftRight,
]);

const isModifier = (key: KeyValues): boolean => {
  return modifierKeySet.has(key);
};

const keyToLabel = (key: KeyValues): string => {
  return KeyLabels[key] || key;
};

const updateKeybind = () => {
  const modifiers = Array.from(activeModifiers);
  const nonModifiers = keybind.value.filter((key) => !isModifier(key));
  keybind.value = [...modifiers, ...nonModifiers];
};

const onBlur = () => {
  isKeybindInputFocused.value = false;
  lastBlurTime.value = Date.now();
  showEmptyKeybindError.value = false;
};

const onFocus = () => {
  isKeybindInputFocused.value = true;
  activeModifiers.clear();
  keybind.value = [];
  showEmptyKeybindError.value = false;
};

const onKeyDown = (event: KeyboardEvent) => {
  const key = event.code as KeyValues;

  if (key === KeyValues.Escape) {
    if (keybindInput.value) {
      keybindInput.value.blur();
    }
    return;
  }

  if (isModifier(key)) {
    activeModifiers.add(key);
  } else if (!keybind.value.includes(key)) {
    keybind.value = keybind.value.filter((k) => isModifier(k));
    keybind.value.push(key);
  }

  updateKeybind();
  showEmptyKeybindError.value = false;
};

const saveKeybind = async () => {
  if (keybind.value.length > 0) {
    await $settings.saveSetting("keybind", JSON.stringify(keybind.value));
    router.push("/");
  } else {
    showEmptyKeybindError.value = true;
  }
};

const toggleAutostart = async () => {
  if (autostart.value === true) {
    await enable();
  } else {
    await disable();
  }
  await $settings.saveSetting("autostart", autostart.value ? "true" : "false");
};

const toggleTypeFilters = async () => {
  try {
    await $settings.saveSetting(
      "show_type_filters",
      typeFiltersOn.value ? "true" : "false"
    );
  } catch (error) {
    console.error("Failed to save type-filter setting:", error);
  }
};

const selectTheme = async (id: string) => {
  if (!isTheme(id)) return;
  currentTheme.value = id;
  applyTheme(id);
  try {
    await $settings.saveSetting("theme", id);
  } catch (error) {
    console.error("Failed to save theme:", error);
  }
};

const loadHistoryInfo = async () => {
  try {
    historyCount.value = await $history.historyCount();
    const stored = await $settings.getSetting("history_limit");
    const parsed = stored ? parseInt(stored, 10) : NaN;
    if (Number.isFinite(parsed) && parsed >= 100 && parsed <= 5000) {
      historyLimit.value = parsed;
    }
  } catch (error) {
    console.error("Failed to load history settings:", error);
  }
};

const saveHistoryLimit = async () => {
  try {
    await $settings.saveSetting("history_limit", String(historyLimit.value));
  } catch (error) {
    console.error("Failed to save history limit:", error);
  }
};

const onClearHistory = async () => {
  if (!clearArmed.value) {
    clearArmed.value = true;
    if (clearTimer) clearTimeout(clearTimer);
    clearTimer = setTimeout(() => {
      clearArmed.value = false;
    }, 3000);
    return;
  }

  if (clearTimer) clearTimeout(clearTimer);
  clearArmed.value = false;
  try {
    await $history.clearHistory();
    router.push("/");
  } catch (error) {
    console.error("Failed to clear history:", error);
  }
};

os.value = platform();

onMounted(async () => {
  // Wipe any handlers left by the previous page, then register ours.
  armKeyboard();

  keyboard.listen([Key.All], (event: KeyboardEvent) => {
    if (isKeybindInputFocused.value) {
      onKeyDown(event);
    }
  }, { prevent: true });

  keyboard.listen([Key.Escape], () => {
    if (isKeybindInputFocused.value) {
      keybindInput.value?.blur();
    } else {
      router.push("/");
    }
  }, { prevent: true });

  switch (os.value) {
    case "macos":
      keyboard.listen([Key.Meta, Key.Enter], () => {
        if (!isKeybindInputFocused.value) {
          saveKeybind();
        }
      }, { prevent: true });
      break;

    case "linux":
    case "windows":
      keyboard.listen([Key.Control, Key.Enter], () => {
        if (!isKeybindInputFocused.value) {
          saveKeybind();
        }
      }, { prevent: true });
      break;
  }

  autostart.value = (await $settings.getSetting("autostart")) === "true";
  typeFiltersOn.value =
    (await $settings.getSetting("show_type_filters")) !== "false";
  const storedTheme = await $settings.getSetting("theme");
  if (storedTheme && isTheme(storedTheme)) {
    currentTheme.value = storedTheme;
    applyTheme(storedTheme);
  }
  await loadHistoryInfo();
});

onUnmounted(() => {
  // No keyboard teardown: the page being navigated to re-establishes the
  // shared keyboard instance in its own onMounted (clear + register).
  // Stopping here could wipe the incoming page's handlers depending on
  // unmount/mount ordering.
  if (clearTimer) clearTimeout(clearTimer);
});
</script>

<style scoped lang="scss">
@use "/styles/settings.scss";
</style>
