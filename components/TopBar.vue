<template>
  <div class="topbar">
    <input
      ref="searchInput"
      v-model="searchQuery"
      @input="onInputChange"
      class="search"
      autocorrect="off"
      autocapitalize="off"
      spellcheck="false"
      type="text"
      placeholder="Type to filter entries..." />
    <button
      v-if="searchQuery"
      class="topbar-action"
      type="button"
      title="Clear search"
      @click="clearSearch">
      <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
        <path
          d="M19 6.41 17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
      </svg>
    </button>
    <button
      class="topbar-action"
      type="button"
      title="Settings"
      @click="openSettings">
      <IconsSettings />
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { navigateTo } from "#app";
import IconsSettings from "~/components/Icons/Settings.vue";

const searchQuery = ref("");
const searchInput = ref<HTMLInputElement | null>(null);

const emit = defineEmits<{
  (e: "search", query: string): void;
  (e: "searchStarted"): void;
  (e: "focus"): void;
}>();

const onInputChange = () => {
  emit("searchStarted");
  emit("search", searchQuery.value);
};

// Clears the query in place (button) instead of hiding the app, which
// Escape does - the list snaps straight back to the full history.
const clearSearch = () => {
  searchQuery.value = "";
  emit("search", "");
  searchInput.value?.focus();
};

const openSettings = () => {
  navigateTo("/settings");
};

defineExpose({ searchInput });
</script>

<style lang="scss">
.topbar {
  width: 100%;
  min-height: 56px;
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  padding-inline: 12px 8px;
  gap: 4px;
  z-index: 100;

  .search {
    flex: 1;
    height: 100%;
    font-size: 18px;
    color: var(--text);
    background-color: transparent;
    outline: none;
    border: none;
    font-family: SFRoundedMedium;
  }

  .topbar-action {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border: none;
    border-radius: 9px;
    background-color: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    transition: background-color 0.2s, color 0.2s;
    flex-shrink: 0;

    svg {
      width: 18px;
      height: 18px;
    }

    &:hover {
      background-color: var(--border);
      color: var(--text);
    }
  }
}
</style>
