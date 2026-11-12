<template>
  <div class="actions-menu" role="menu">
    <button
      v-for="action in actions"
      :key="action.id"
      class="actions-item"
      :class="{ disabled: !action.available }"
      role="menuitem"
      :disabled="!action.available"
      @click="$emit('action', action.id)">
      <component :is="action.icon" class="actions-icon" :filled="action.filled" />
      <span class="actions-label">{{ action.label }}</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import IconsCopy from "~/components/Icons/Copy.vue";
import IconsPin from "~/components/Icons/Pin.vue";
import IconsDelete from "~/components/Icons/Delete.vue";

interface ActionItem {
  id: string;
  label: string;
  icon: any;
  available: boolean;
  filled?: boolean;
}

const props = defineProps<{
  pinned: boolean;
}>();

const emit = defineEmits<{
  (e: "action", id: string): void;
}>();

const actions = computed<ActionItem[]>(() => [
  {
    id: "copy",
    label: "Copy to clipboard",
    icon: IconsCopy,
    available: true,
  },
  {
    id: "pin",
    label: props.pinned ? "Unpin" : "Pin",
    icon: IconsPin,
    available: true,
    filled: props.pinned,
  },
  {
    id: "delete",
    label: "Delete",
    icon: IconsDelete,
    available: true,
  },
]);
</script>

<style scoped lang="scss">
.actions-menu {
  position: absolute;
  bottom: 46px;
  right: 6px;
  width: 172px;
  background-color: var(--background);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 5px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  z-index: 200;
  box-shadow: 0 10px 28px rgba(0, 0, 0, 0.45);
  animation: menu-in 0.12s ease-out;

  @keyframes menu-in {
    from {
      opacity: 0;
      transform: translateY(5px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .actions-item {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    padding: 7px 9px;
    border: none;
    border-radius: 7px;
    background: transparent;
    color: var(--text);
    font-family: SFRoundedMedium;
    font-size: 12px;
    text-align: left;
    cursor: pointer;
    transition: background-color 0.15s;

    .actions-icon {
      width: 15px;
      height: 15px;
      color: var(--text-secondary);
      flex-shrink: 0;
    }

    &:hover:not(.disabled) {
      background-color: var(--border);

      .actions-icon {
        color: var(--text);
      }
    }

    &.disabled {
      opacity: 0.4;
      cursor: default;
    }
  }
}
</style>
