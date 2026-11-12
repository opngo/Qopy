<template>
  <div
    :class="[
      'result',
      {
        selected,
        tagged: Boolean(item.tag_color),
        codeRow: item.content_type === ContentType.Code,
      },
    ]"
    :style="rowStyle"
    @click="$emit('select')"
    @contextmenu.prevent="$emit('contextmenu', item, $event)"
    :ref="el => { if (selected && el) $emit('setRef', el as HTMLElement) }">
    <template v-if="item.content_type === 'image'">
      <img
        v-if="imageUrl"
        :src="imageUrl"
        alt="Image"
        class="image"
        @error="$emit('imageError')" />
      <IconsImage v-else class="icon" />
    </template>
    <template v-else-if="hasFavicon(item.favicon ?? '') && !faviconFailed">
      <img
        :src="getFaviconFromDb(item.favicon!)"
        alt="Favicon"
        class="favicon"
        @error="faviconFailed = true" />
    </template>
    <IconsLink
      v-else-if="item.content_type === ContentType.Link || hasFavicon(item.favicon ?? '')"
      class="icon" />
    <template v-else-if="item.content_type === ContentType.File">
      <img
        v-if="imageUrl"
        :src="imageUrl"
        alt="File preview"
        class="image"
        @error="$emit('previewError', item)" />
      <IconsFile v-else class="icon" />
    </template>
    <IconsText
      class="icon"
      v-else-if="item.content_type === ContentType.Text" />
    <svg
      v-else-if="item.content_type === ContentType.Color"
      width="18"
      height="18"
      viewBox="0 0 18 18"
      fill="none"
      xmlns="http://www.w3.org/2000/svg">
      <g>
        <rect width="18" height="18" />
        <path
          d="M9 18C12.2154 18 15.1865 16.2846 16.7942 13.5C18.4019 10.7154 18.4019 7.28461 16.7942 4.5C15.1865 1.71539 12.2154 -1.22615e-06 9 0C5.78461 0 2.81347 1.71539 1.20577 4.5C-0.401925 7.28461 -0.401923 10.7154 1.20577 13.5C2.81347 16.2846 5.78461 18 9 18Z"
          fill="var(--icon, #E5DFD5)" />
        <path
          d="M9 16C7.14348 16 5.36301 15.2625 4.05025 13.9497C2.7375 12.637 2 10.8565 2 9C2 7.14348 2.7375 5.36301 4.05025 4.05025C5.36301 2.7375 7.14348 2 9 2C10.8565 2 12.637 2.7375 13.9497 4.05025C15.2625 5.36301 16 7.14348 16 9C16 10.8565 15.2625 12.637 13.9497 13.9497C12.637 15.2625 10.8565 16 9 16Z"
          :fill="item.content" />
      </g>
    </svg>
    <IconsCode
      class="icon"
      v-else-if="item.content_type === ContentType.Code" />
    <IconsPin v-if="item.pinned" class="pin-indicator" :filled="true" />
    <span class="label" v-if="item.content_type === ContentType.Image">
      Image ({{ dimensions || "Loading..." }})
    </span>
    <span class="label" v-else>{{ displayLabel }}</span>
    <span class="type-pill">{{ typePill }}</span>
    <span class="row-actions" @click.stop>
      <button
        class="row-action"
        type="button"
        :class="{ active: item.pinned }"
        :title="item.pinned ? 'Unpin' : 'Pin'"
        @click.stop="$emit('pin', item)">
        <IconsPin :filled="item.pinned" />
      </button>
      <button
        class="row-action"
        type="button"
        title="Delete"
        @click.stop="$emit('delete', item)">
        <IconsDelete />
      </button>
    </span>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { ContentType } from "~/types/types";
import type { HistoryItem } from "~/types/types";
import IconsPin from "~/components/Icons/Pin.vue";
import IconsDelete from "~/components/Icons/Delete.vue";

const props = defineProps<{
  item: HistoryItem;
  selected: boolean;
  imageUrl?: string;
  dimensions?: string;
}>();

defineEmits<{
  (e: "select"): void;
  (e: "imageError"): void;
  (e: "previewError", item: HistoryItem): void;
  (e: "setRef", el: HTMLElement): void;
  (e: "pin", item: HistoryItem): void;
  (e: "delete", item: HistoryItem): void;
  (e: "contextmenu", item: HistoryItem, event: MouseEvent): void;
}>();

const faviconFailed = ref(false);

const hasFavicon = (str: string): boolean => {
  return str.trim() !== "";
};

const getFaviconFromDb = (favicon: string): string => {
  return `data:image/png;base64,${favicon}`;
};

// --- Per-entry type pill (txt, png, url, json, py, ...) --------------------

const LANG_COLORS: Record<string, string> = {
  python: "#4b8bbe",
  javascript: "#c8b458",
  typescript: "#5195c9",
  rust: "#e88a5d",
  bash: "#7fb069",
  shell: "#7fb069",
  go: "#58c4dc",
  java: "#c98a4b",
  c: "#a8a29a",
  cpp: "#7fb0d9",
  csharp: "#5aa86a",
  html: "#e0603f",
  css: "#9b7fd4",
  json: "#d9b45b",
  sql: "#d98a5b",
  yaml: "#d96b6b",
  markdown: "#6b8fd4",
  php: "#9a94c9",
  ruby: "#d96b8a",
  swift: "#e0705a",
  kotlin: "#b08fd9",
};

const language = computed<string>(() =>
  (props.item.language ?? "").toLowerCase().trim()
);
const languageColor = computed<string>(
  () => LANG_COLORS[language.value] ?? "var(--accent)"
);

// Row style: `--deco` carries the language color for the code-row gradient
// (language color on the left fading to the theme's default on the right);
// `--tag` carries the color tag.
const rowStyle = computed<Record<string, string>>(() => {
  const style: Record<string, string> = {};
  if (props.item.tag_color) style["--tag"] = props.item.tag_color;
  if (props.item.content_type === ContentType.Code) {
    style["--deco"] = languageColor.value;
  }
  return style;
});

const typePill = computed<string>(() => {
  switch (props.item.content_type) {
    case ContentType.Code:
      return props.item.language ?? "code";
    case ContentType.Text:
      return "txt";
    case ContentType.Image:
      return "png";
    case ContentType.Link:
      return "url";
    case ContentType.Color:
      return "hex";
    case ContentType.File: {
      const ext = (props.item.content.split(".").pop() ?? "").toLowerCase();
      return ext && ext.length <= 5 ? ext : "file";
    }
    default:
      return "txt";
  }
});

// --- Row label ---------------------------------------------------------------
// Code entries: the first line that is not just braces/punctuation (a JSON
// file no longer displays as a lone "{"); otherwise the language name.

const truncate = (s: string, maxChars: number): string => {
  return s.length > maxChars ? s.slice(0, maxChars - 3) + "..." : s;
};

const displayLabel = computed<string>(() => {
  const content = props.item.content;
  if (props.item.content_type === ContentType.Code) {
    const lines = content.split("\n").map((l) => l.trim());
    const meaningful = lines.find(
      (l) => l.length > 0 && !/^[{}[\],:\s]+$/.test(l)
    );
    if (meaningful) return truncate(meaningful, 26);
    return (props.item.language ?? "code").toUpperCase();
  }
  const singleLine = content.split("\n")[0] ?? "";
  return truncate(singleLine, 26);
});
</script>

<style scoped lang="scss">
.result {
  display: flex;
  gap: 12px;
  padding: 11px 8px 11px 11px;
  border-radius: 10px;
  cursor: pointer;
  align-items: center;
  position: relative;
  border: 1px solid transparent;

  &.selected {
    background-color: var(--selection);
  }

  // Code rows: the row itself is the language color on the left, fading to
  // the theme's default color toward the right (the transparent part of the
  // gradient lets the normal/selected/tagged background show through).
  &.code-row {
    background-image: linear-gradient(
      to right,
      var(--deco) 0%,
      transparent 80%
    );
  }

  // Color tag: the whole row gets highlighted in the tag color.
  &.tagged {
    border-color: var(--tag);
    background-color: color-mix(in srgb, var(--tag) 16%, transparent);

    &.selected {
      background-color: color-mix(in srgb, var(--tag) 30%, var(--selection));
    }
  }

  .favicon,
  .image,
  .icon,
  .pin-indicator {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
    position: relative;
    z-index: 1;
  }

  .pin-indicator {
    color: var(--accent);
    margin-left: -8px;
    margin-right: -4px;
  }

  span.label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text);
    min-width: 0;
    position: relative;
    z-index: 1;
  }

  // Same default styling for every pill (including code languages).
  .type-pill {
    flex-shrink: 0;
    position: relative;
    z-index: 1;
    font-family: CommitMono;
    font-size: 10px;
    line-height: 1;
    letter-spacing: 0.4px;
    color: var(--text-muted);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 3px 6px;
  }

  .row-actions {
    display: flex;
    align-items: center;
    gap: 1px;
    margin-left: auto;
    flex-shrink: 0;
    opacity: 0;
    transition: opacity 0.15s;
    position: relative;
    z-index: 2;

    .result.selected & {
      opacity: 1;
    }
  }

  &:hover .row-actions {
    opacity: 1;
  }

  .row-action {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    transition: background-color 0.15s, color 0.15s;

    svg {
      width: 15px;
      height: 15px;
    }

    &:hover {
      background-color: var(--border);
      color: var(--text);
    }

    &.active {
      color: var(--accent);
    }

    &.active:hover {
      color: var(--text);
    }
  }
}
</style>
