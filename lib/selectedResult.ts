import type { ComputedRef } from "vue";
import type { HistoryItem } from "~/types/types";

export interface GroupedHistory {
  label: string;
  items: HistoryItem[];
}

// Selection is tracked by item id (stable) instead of group/item indexes.
// Indexes shift when new items are prepended or chunks are appended,
// which used to silently move the selection to the wrong item.
export const selectedItemId = ref<string | null>(null);
export const selectedElement = ref<HTMLElement | null>(null);

export const useSelectedResult = (
  groupedHistory: ComputedRef<GroupedHistory[]>
) => {
  const flatItems = computed<HistoryItem[]>(() =>
    groupedHistory.value.flatMap((group) => group.items)
  );

  const selectedItem = computed<HistoryItem | null>(() => {
    if (!selectedItemId.value) return null;
    return (
      flatItems.value.find((item) => item.id === selectedItemId.value) ?? null
    );
  });

  const selectedFlatIndex = computed(() =>
    flatItems.value.findIndex((item) => item.id === selectedItemId.value)
  );

  const isSelected = (item: HistoryItem): boolean => {
    return item.id === selectedItemId.value;
  };

  const positionOfItem = (
    item: HistoryItem
  ): { groupIndex: number; itemIndex: number } | null => {
    for (let groupIndex = 0; groupIndex < groupedHistory.value.length; groupIndex++) {
      const items = groupedHistory.value[groupIndex].items;
      const itemIndex = items.findIndex((i) => i.id === item.id);
      if (itemIndex !== -1) return { groupIndex, itemIndex };
    }
    return null;
  };

  const selectItem = (id: string | null): void => {
    selectedItemId.value = id;
  };

  const selectFirst = (): void => {
    selectedItemId.value = flatItems.value[0]?.id ?? null;
  };

  const selectNext = (): void => {
    const index = selectedFlatIndex.value;
    if (index === -1) return selectFirst();
    const next = flatItems.value[index + 1];
    if (next) selectedItemId.value = next.id;
  };

  const selectPrevious = (): void => {
    const index = selectedFlatIndex.value;
    if (index <= 0) return;
    selectedItemId.value = flatItems.value[index - 1].id;
  };

  // The item to select after `item` disappears (delete/unpin).
  const neighborOf = (item: HistoryItem): HistoryItem | null => {
    const index = flatItems.value.findIndex((i) => i.id === item.id);
    if (index === -1) return flatItems.value[0] ?? null;
    return flatItems.value[index + 1] ?? flatItems.value[index - 1] ?? null;
  };

  return {
    selectedItem,
    isSelected,
    selectItem,
    selectFirst,
    selectNext,
    selectPrevious,
    positionOfItem,
    neighborOf,
    flatItems,
    selectedElement,
  };
};
