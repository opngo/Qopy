<template>
  <div
    class="tag-menu"
    role="menu"
    :style="{ left: x + 'px', top: y + 'px' }">
    <div class="tag-menu-title">Tag entry</div>
    <div class="tag-swatches">
      <button
        v-for="color in TAG_COLORS"
        :key="color"
        type="button"
        class="tag-swatch"
        :class="{ active: color === current }"
        :style="{ background: color }"
        :title="color"
        @click="$emit('select', color)" />
      <button
        type="button"
        class="tag-swatch clear"
        :class="{ active: !current }"
        title="Remove tag"
        @click="$emit('select', null)">
        <svg
          viewBox="0 0 24 24"
          width="12"
          height="12"
          fill="currentColor"
          aria-hidden="true">
          <path
            d="M19 6.41 17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" />
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { TAG_COLORS } from "~/lib/tagColors";

defineProps<{
  x: number;
  y: number;
  current?: string | null;
}>();

defineEmits<{
  (e: "select", color: string | null): void;
}>();
</script>

<style scoped lang="scss">
.tag-menu {
  position: fixed;
  z-index: 300;
  width: 196px;
  background-color: var(--background);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 8px;
  box-shadow: 0 10px 28px rgba(0, 0, 0, 0.45);
  animation: tag-menu-in 0.1s ease-out;

  @keyframes tag-menu-in {
    from {
      opacity: 0;
      transform: translateY(4px) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .tag-menu-title {
    font-family: SFRoundedSemiBold;
    font-size: 10px;
    letter-spacing: 0.8px;
    text-transform: uppercase;
    color: var(--text-muted);
    padding: 0 2px 7px;
  }

  .tag-swatches {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .tag-swatch {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: transform 0.1s, border-color 0.1s;

    &:hover {
      transform: scale(1.15);
    }

    &.active {
      border-color: var(--text);
    }

    &.clear {
      background: transparent;
      border: 1px solid var(--border);
      color: var(--text-muted);

      &.active {
        border-color: var(--text);
        color: var(--text);
      }
    }
  }
}
</style>
