<template>
  <main @contextmenu.prevent>
    <TopBar ref="topBar" @search="searchHistory" @searchStarted="searchStarted" />
    <div
      class="filter-bar"
      v-if="history.length > 0 && showTypeFilters">
      <button
        v-for="f in typeFilters"
        :key="f.value"
        class="filter-chip"
        :class="{ active: typeFilter === f.value }"
        @click="typeFilter = f.value"
      >
        {{ f.label }}
      </button>
    </div>
    <div class="container">
      <OverlayScrollbarsComponent class="results" ref="resultsContainer"
        :options="{ scrollbars: { autoHide: 'scroll' } }">
        <div v-for="(group, groupIndex) in groupedHistory" :key="groupIndex" class="group">
          <div class="time-separator">{{ group.label }}</div>
          <div class="results-group">
            <Result v-for="item in group.items" :key="item.id" :item="item"
              :selected="isSelected(item)" :image-url="imageUrls[item.id]"
              :dimensions="imageDimensions[item.id]" @select="selectItem(item.id)" @image-error="onImageError"
              @preview-error="onPreviewError"
              @pin="togglePin" @delete="deleteItem"
              @contextmenu="onItemContextMenu"
              @setRef="(el) => (selectedElement = el)" />
          </div>
        </div>
        <div v-if="groupedHistory.length === 0" class="empty-state">
          <p class="empty-title">
            {{ searchQuery ? "No results" : typeFilter !== "all" ? `No ${typeFilter} items` : "No items yet" }}
          </p>
          <p class="empty-sub">
            {{
              searchQuery
                ? `Nothing matches "${searchQuery}"`
                : typeFilter !== "all"
                  ? "Try another filter or copy something"
                  : "Copy something and it will show up here"
            }}
          </p>
        </div>
      </OverlayScrollbarsComponent>
      <div class="right">
        <span class="preview-lang" v-if="previewLanguage">{{ previewLanguage }}</span>
        <div class="content" v-if="selectedItem?.content_type === ContentType.Image">
          <img :src="imageUrls[selectedItem.id]" alt="Image" class="image" />
        </div>
        <div v-else-if="selectedItem && isYoutubeWatchUrl(selectedItem.content)" class="content">
          <img class="image" :src="getYoutubeThumbnail(selectedItem.content)" alt="YouTube Thumbnail" />
        </div>
        <div class="content" v-else-if="
          selectedItem?.content_type === ContentType.Link && pageOgImage
        ">
          <img :src="pageOgImage" alt="Image" class="image" />
        </div>
        <div
          class="content"
          v-else-if="
            selectedItem?.content_type === ContentType.File &&
            imageUrls[selectedItem.id]
          ">
          <img :src="imageUrls[selectedItem.id]" alt="File preview" class="image" />
        </div>
        <OverlayScrollbarsComponent v-else class="content">
          <pre v-if="highlightedCode" class="code-preview"
            ><code v-html="highlightedCode" /></pre
          >
          <span v-else class="content-text">{{ selectedItem?.content || "" }}</span>
        </OverlayScrollbarsComponent>
        <div v-if="toast" class="toast">{{ toast }}</div>
        <OverlayScrollbarsComponent class="information" :options="{ scrollbars: { autoHide: 'scroll' } }">
          <div class="title">Information</div>
          <div class="info-content" v-if="selectedItem && getInfo">
            <div class="info-row" v-for="(row, index) in infoRows" :key="index">
              <p class="label">{{ row.label }}</p>
              <span :class="{ 'url-truncate': row.isUrl }" :data-text="row.value">
                <img v-if="row.icon" :src="row.icon" :alt="String(row.value)">
                {{ row.value }}
              </span>
            </div>
          </div>
          <div v-else class="info-empty">Nothing selected</div>
        </OverlayScrollbarsComponent>
      </div>
    </div>
    <ActionsMenu
      v-if="showActionsMenu && selectedItem"
      :pinned="selectedItem.pinned"
      @action="onActionsMenuAction" />
    <!-- Custom right-click tag menu (native context menu is suppressed) -->
    <div
      v-if="tagMenu"
      class="tag-menu-backdrop"
      @click="tagMenu = null" />
    <TagMenu
      v-if="tagMenu"
      :x="tagMenu.x"
      :y="tagMenu.y"
      :current="tagMenu.item.tag_color"
      @select="onTagSelect" />
    <BottomBar :primary-action="{
      text: 'Paste',
      icon: IconsEnter,
      onClick: pasteSelectedItem,
    }" :secondary-action="selectedItem ? {
      text: 'Actions',
      icon: IconsK,
      showModifier: true,
      onClick: toggleActionsMenu,
    } : undefined" />
  </main>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from "vue";
import { OverlayScrollbarsComponent } from "overlayscrollbars-vue";
import "overlayscrollbars/overlayscrollbars.css";
import { app, window } from "@tauri-apps/api";
import { platform } from "@tauri-apps/plugin-os";
import { listen } from "@tauri-apps/api/event";
import { useNuxtApp } from "#app";
import { invoke } from "@tauri-apps/api/core";
import { HistoryItem, ContentType } from "~/types/types";
import type {
  InfoText,
  InfoImage,
  InfoFile,
  InfoLink,
  InfoColor,
  InfoCode,
} from "~/types/types";
import {
  selectedItemId,
  selectedElement,
  useSelectedResult,
} from "~/lib/selectedResult";
import { appKeyboard, armKeyboard } from "~/lib/keyboard";
import { beginPaste } from "~/lib/appState";
import IconsEnter from "~/components/Icons/Enter.vue";
import IconsK from "~/components/Icons/K.vue";
import ActionsMenu from "~/components/ActionsMenu.vue";
import TagMenu from "~/components/TagMenu.vue";
import { Key } from "@waradu/keyboard";
import hljs from "highlight.js/lib/core";
import bash from "highlight.js/lib/languages/bash";
import c from "highlight.js/lib/languages/c";
import cpp from "highlight.js/lib/languages/cpp";
import csharp from "highlight.js/lib/languages/csharp";
import css from "highlight.js/lib/languages/css";
import dart from "highlight.js/lib/languages/dart";
import diff from "highlight.js/lib/languages/diff";
import dockerfile from "highlight.js/lib/languages/dockerfile";
import elixir from "highlight.js/lib/languages/elixir";
import fsharp from "highlight.js/lib/languages/fsharp";
import go from "highlight.js/lib/languages/go";
import graphql from "highlight.js/lib/languages/graphql";
import groovy from "highlight.js/lib/languages/groovy";
import haskell from "highlight.js/lib/languages/haskell";
import ini from "highlight.js/lib/languages/ini";
import java from "highlight.js/lib/languages/java";
import javascript from "highlight.js/lib/languages/javascript";
import json from "highlight.js/lib/languages/json";
import kotlin from "highlight.js/lib/languages/kotlin";
import less from "highlight.js/lib/languages/less";
import lua from "highlight.js/lib/languages/lua";
import makefile from "highlight.js/lib/languages/makefile";
import markdown from "highlight.js/lib/languages/markdown";
import nginx from "highlight.js/lib/languages/nginx";
import objectivec from "highlight.js/lib/languages/objectivec";
import perl from "highlight.js/lib/languages/perl";
import php from "highlight.js/lib/languages/php";
import powershell from "highlight.js/lib/languages/powershell";
import protobuf from "highlight.js/lib/languages/protobuf";
import python from "highlight.js/lib/languages/python";
import r from "highlight.js/lib/languages/r";
import ruby from "highlight.js/lib/languages/ruby";
import rust from "highlight.js/lib/languages/rust";
import scala from "highlight.js/lib/languages/scala";
import scss from "highlight.js/lib/languages/scss";
import sql from "highlight.js/lib/languages/sql";
import swift from "highlight.js/lib/languages/swift";
import typescript from "highlight.js/lib/languages/typescript";
import xmlLang from "highlight.js/lib/languages/xml";
import yaml from "highlight.js/lib/languages/yaml";

hljs.registerLanguage("bash", bash);
hljs.registerLanguage("shell", bash);
hljs.registerLanguage("sh", bash);
hljs.registerLanguage("c", c);
hljs.registerLanguage("cpp", cpp);
hljs.registerLanguage("c++", cpp);
hljs.registerLanguage("csharp", csharp);
hljs.registerLanguage("c#", csharp);
hljs.registerLanguage("css", css);
hljs.registerLanguage("dart", dart);
hljs.registerLanguage("diff", diff);
hljs.registerLanguage("dockerfile", dockerfile);
hljs.registerLanguage("docker", dockerfile);
hljs.registerLanguage("elixir", elixir);
hljs.registerLanguage("fsharp", fsharp);
hljs.registerLanguage("f#", fsharp);
hljs.registerLanguage("go", go);
hljs.registerLanguage("graphql", graphql);
hljs.registerLanguage("groovy", groovy);
hljs.registerLanguage("haskell", haskell);
hljs.registerLanguage("ini", ini);
hljs.registerLanguage("properties", ini);
hljs.registerLanguage("conf", ini);
hljs.registerLanguage("java", java);
hljs.registerLanguage("javascript", javascript);
hljs.registerLanguage("js", javascript);
hljs.registerLanguage("json", json);
hljs.registerLanguage("kotlin", kotlin);
hljs.registerLanguage("kt", kotlin);
hljs.registerLanguage("less", less);
hljs.registerLanguage("lua", lua);
hljs.registerLanguage("makefile", makefile);
hljs.registerLanguage("make", makefile);
hljs.registerLanguage("markdown", markdown);
hljs.registerLanguage("md", markdown);
hljs.registerLanguage("nginx", nginx);
hljs.registerLanguage("objective-c", objectivec);
hljs.registerLanguage("objc", objectivec);
hljs.registerLanguage("perl", perl);
hljs.registerLanguage("php", php);
hljs.registerLanguage("powershell", powershell);
hljs.registerLanguage("ps1", powershell);
hljs.registerLanguage("protobuf", protobuf);
hljs.registerLanguage("proto", protobuf);
hljs.registerLanguage("python", python);
hljs.registerLanguage("py", python);
hljs.registerLanguage("r", r);
hljs.registerLanguage("ruby", ruby);
hljs.registerLanguage("rb", ruby);
hljs.registerLanguage("rust", rust);
hljs.registerLanguage("rs", rust);
hljs.registerLanguage("scala", scala);
hljs.registerLanguage("scss", scss);
hljs.registerLanguage("sql", sql);
hljs.registerLanguage("swift", swift);
hljs.registerLanguage("typescript", typescript);
hljs.registerLanguage("ts", typescript);
hljs.registerLanguage("xml", xmlLang);
hljs.registerLanguage("html", xmlLang);
hljs.registerLanguage("yaml", yaml);
hljs.registerLanguage("yml", yaml);

interface GroupedHistory {
  label: string;
  items: HistoryItem[];
}

const { $history, $settings } = useNuxtApp();

// Shared across all page instances - guarantees exactly one set of key
// handlers no matter how many times this page remounts (see lib/keyboard.ts).
const keyboard = appKeyboard;

const CHUNK_SIZE = 50;
const SCROLL_THRESHOLD = 100;
const SCROLL_PADDING = 8;
const TOP_SCROLL_PADDING = 37;

const history = ref<HistoryItem[]>([]);
let offset = 0;
let isLoading = false;

const resultsContainer = shallowRef<InstanceType<
  typeof OverlayScrollbarsComponent
> | null>(null);
const searchQuery = ref("");
const os = ref<string>("");

// Quick type filters (chip bar under the search bar)
const typeFilter = ref("all");
const typeFilters = [
  { value: "all", label: "All" },
  { value: "text", label: "Text" },
  { value: "code", label: "Code" },
  { value: "link", label: "Link" },
  { value: "image", label: "Image" },
  { value: "color", label: "Color" },
  { value: "file", label: "File" },
];
const filterTypeMap: Record<string, string[]> = {
  text: ["text"],
  code: ["code"],
  link: ["link", "url"],
  image: ["image", "image_file"],
  color: ["color"],
  file: ["file", "file_url"],
};
const matchesFilter = (item: { content_type: string }): boolean => {
  if (typeFilter.value === "all") return true;
  const types = filterTypeMap[typeFilter.value];
  return types ? types.includes(item.content_type) : true;
};
// Plain `ref` (deep reactive), NOT shallowRef: thumbnails are assigned per
// item id after the list renders, and with a shallowRef those updates never
// reach the component - every row kept showing whatever was painted first.
const imageUrls = ref<Record<string, string>>({});
const imageDimensions = ref<Record<string, string>>({});
const imageSizes = ref<Record<string, string>>({});
const lastUpdateTime = ref<number>(Date.now());
const pageTitle = ref<string>("");
const pageOgImage = ref<string>("");
const showActionsMenu = ref(false);
const toast = ref("");
// Type-filter chip bar can be turned off in Settings (default: on).
const showTypeFilters = ref(true);
// Custom right-click context menu state (tag an entry with a color).
const tagMenu = ref<{ item: HistoryItem; x: number; y: number } | null>(null);

const topBar = ref<{ searchInput: HTMLInputElement | null } | null>(null);

let toastTimer: ReturnType<typeof setTimeout> | null = null;
const showToast = (message: string): void => {
  toast.value = message;
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    toast.value = "";
  }, 1100);
};

const isSameDay = (date1: Date, date2: Date): boolean => {
  return (
    date1.getFullYear() === date2.getFullYear() &&
    date1.getMonth() === date2.getMonth() &&
    date1.getDate() === date2.getDate()
  );
};

const getWeekNumber = (date: Date): number => {
  const firstDayOfYear = new Date(date.getFullYear(), 0, 1);
  return Math.ceil(
    ((date.getTime() - firstDayOfYear.getTime()) / 86400000 +
      firstDayOfYear.getDay() +
      1) /
      7
  );
};

const groupedHistory = computed<GroupedHistory[]>(() => {
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const thisWeek = getWeekNumber(now);
  const thisYear = now.getFullYear();

  const groups: Record<string, HistoryItem[]> = {
    Today: [],
    Yesterday: [],
    "This Week": [],
    "Last Week": [],
    "This Year": [],
    "Last Year": [],
  };

  const filteredItems = history.value.filter((item) => {
    if (!matchesFilter(item)) return false;
    if (searchQuery.value && !item.content.toLowerCase().includes(searchQuery.value.toLowerCase())) {
      return false;
    }
    return true;
  });

  const yesterday = new Date(today.getTime() - 86400000);

  filteredItems.forEach((item) => {
    if (item.pinned) return; // pinned items get their own group below

    const itemDate = new Date(item.timestamp);
    const itemWeek = getWeekNumber(itemDate);
    const itemYear = itemDate.getFullYear();

    if (isSameDay(itemDate, today)) groups.Today.push(item);
    else if (isSameDay(itemDate, yesterday)) groups.Yesterday.push(item);
    else if (itemYear === thisYear && itemWeek === thisWeek)
      groups["This Week"].push(item);
    else if (itemYear === thisYear && itemWeek === thisWeek - 1)
      groups["Last Week"].push(item);
    else if (itemYear === thisYear) groups["This Year"].push(item);
    else groups["Last Year"].push(item);
  });

  const result: GroupedHistory[] = [];
  const pinned = filteredItems.filter((item) => item.pinned);
  if (pinned.length > 0) result.push({ label: "Pinned", items: pinned });
  result.push(
    ...Object.entries(groups)
      .filter(([_, items]) => items.length > 0)
      .map(([label, items]) => ({ label, items }))
  );
  return result;
});

const {
  selectedItem,
  isSelected,
  selectItem,
  selectFirst,
  selectNext,
  selectPrevious,
  positionOfItem,
  neighborOf,
} = useSelectedResult(groupedHistory);

const mapToHistoryItem = (raw: {
  id: string;
  source: string;
  source_icon?: string | null;
  content_type: string;
  content: string;
  favicon?: string | null;
  timestamp: string | Date;
  language?: string | null;
  pinned?: boolean | number | null;
  tag_color?: string | null;
}): HistoryItem => {
  const item = new HistoryItem(
    raw.source,
    raw.content_type as ContentType,
    raw.content,
    raw.favicon ?? undefined,
    raw.source_icon ?? undefined,
    raw.language ?? undefined,
    Boolean(raw.pinned)
  );
  item.id = raw.id;
  item.timestamp = new Date(raw.timestamp);
  item.tag_color = raw.tag_color ?? undefined;
  return item;
};

const IMAGE_FILE_MIME: Record<string, string> = {
  svg: "image/svg+xml",
  png: "image/png",
  jpg: "image/jpeg",
  jpeg: "image/jpeg",
  gif: "image/gif",
  webp: "image/webp",
  bmp: "image/bmp",
  avif: "image/avif",
  ico: "image/x-icon",
};

const isImageFile = (path: string): boolean => {
  const ext = (path.split(".").pop() ?? "").toLowerCase();
  return ext in IMAGE_FILE_MIME;
};

const processItem = async (raw: {
  id: string;
  source: string;
  source_icon?: string | null;
  content_type: string;
  content: string;
  favicon?: string | null;
  timestamp: string | Date;
  language?: string | null;
  pinned?: boolean | number | null;
  tag_color?: string | null;
}): Promise<HistoryItem> => {
  const historyItem = mapToHistoryItem(raw);

  // File entries that point at an image (SVG, PNG, ...) get a real preview
  // thumbnail instead of a generic file icon.
  if (historyItem.content_type === ContentType.File && isImageFile(historyItem.content)) {
    try {
      const base64 = await $history.readImage({
        filename: historyItem.content,
      });
      const ext = (historyItem.content.split(".").pop() ?? "").toLowerCase();
      imageUrls.value[historyItem.id] = `data:${IMAGE_FILE_MIME[ext]};base64,${base64}`;
    } catch (error) {
      // File missing/unreadable - keep the generic file icon.
      console.warn("Could not load file preview:", historyItem.content);
    }
    return historyItem;
  }

  if (historyItem.content_type === ContentType.Image) {
    try {
      const base64 = await $history.readImage({
        filename: historyItem.content,
      });
      const size = Math.ceil((base64.length * 3) / 4);
      imageSizes.value[historyItem.id] = formatFileSize(size);

      const img = new Image();
      img.src = `data:image/png;base64,${base64}`;
      imageUrls.value[historyItem.id] = img.src;

      await new Promise<void>((resolve) => {
        img.onload = () => {
          imageDimensions.value[
            historyItem.id
          ] = `${img.width}x${img.height}`;
          resolve();
        };
        img.onerror = () => {
          imageDimensions.value[historyItem.id] = "Error";
          resolve();
        };
      });
    } catch (error) {
      console.error("Error processing image:", error);
      imageDimensions.value[historyItem.id] = "Error";
      imageSizes.value[historyItem.id] = "Error";
    }
  }
  return historyItem;
};

const loadHistoryChunk = async (): Promise<void> => {
  if (isLoading) return;
  isLoading = true;

  try {
    const results = await $history.loadHistoryChunk(offset, CHUNK_SIZE);
    if (!results.length) {
      isLoading = false;
      return;
    }

    const processedItems = await Promise.all(
      results.map((raw) => processItem(raw))
    );

    // Appending older items must never touch the current selection.
    history.value = [...history.value, ...processedItems];
    offset += CHUNK_SIZE;
  } catch (error) {
    console.error("Failed to load history:", error);
  } finally {
    isLoading = false;
  }
};

const handleScroll = (): void => {
  const viewport = resultsContainer.value?.osInstance()?.elements().viewport;
  if (!viewport) return;

  const { scrollTop, scrollHeight, clientHeight } = viewport;
  if (scrollHeight - scrollTop - clientHeight < SCROLL_THRESHOLD) {
    loadHistoryChunk();
  }
};

const scrollToSelectedItem = (): void => {
  nextTick(() => {
    setTimeout(() => {
      const viewport = resultsContainer.value?.osInstance()?.elements().viewport;
      const el = selectedElement.value;
      if (!el || !el.isConnected || !viewport) return;

      const viewportRect = viewport.getBoundingClientRect();
      const elementRect = el.getBoundingClientRect();

      const position = selectedItem.value
        ? positionOfItem(selectedItem.value)
        : null;
      const isFirstItemInGroup = position?.itemIndex === 0;
      const isAbove = elementRect.top < viewportRect.top + SCROLL_PADDING;
      const isBelow = elementRect.bottom > viewportRect.bottom - SCROLL_PADDING;

      if (isAbove) {
        viewport.scrollTo({
          top:
            viewport.scrollTop +
            (elementRect.top - viewportRect.top) -
            (isFirstItemInGroup ? TOP_SCROLL_PADDING : SCROLL_PADDING),
          behavior: "smooth",
        });
      } else if (isBelow) {
        viewport.scrollTo({
          top:
            viewport.scrollTop +
            (elementRect.bottom - viewportRect.bottom) +
            SCROLL_PADDING,
          behavior: "smooth",
        });
      }
    }, 10);
  });
};

const scrollToTop = (): void => {
  const viewport = resultsContainer.value?.osInstance()?.elements().viewport;
  viewport?.scrollTo({ top: 0, behavior: "smooth" });
};

let searchController: AbortController | null = null;
let searchQueue: Array<string> = [];
let isProcessingSearch = false;

const searchStarted = () => {
  if (searchController) {
    searchController.abort();
  }
};

const processSearchQueue = async () => {
  if (isProcessingSearch || searchQueue.length === 0) return;

  isProcessingSearch = true;
  const query = searchQueue.pop();
  searchQueue = [];

  try {
    if (!query || !query.trim()) {
      searchQuery.value = "";
      history.value = [];
      offset = 0;
      await loadHistoryChunk();
      selectFirst();
      isProcessingSearch = false;
      return;
    }

    const results = await $history.searchHistory(query);

    if (searchController?.signal.aborted) {
      isProcessingSearch = false;
      return;
    }

    history.value = results.map((raw) => mapToHistoryItem(raw));
    if (history.value.length > 0) {
      selectFirst();
    } else {
      selectItem(null);
    }
  } catch (error) {
    console.error("Search error:", error);
  } finally {
    isProcessingSearch = false;
    if (searchQueue.length > 0) {
      requestAnimationFrame(() => processSearchQueue());
    }
  }
};

const searchHistory = (query: string): void => {
  searchQuery.value = query;

  if (searchController) {
    searchController.abort();
  }

  searchController = new AbortController();

  searchQueue.push(query);
  if (!isProcessingSearch) {
    processSearchQueue();
  }
};

// Re-sync the loaded items with the database. Returns the id of the newest
// item when a brand-new entry arrived, otherwise null.
const updateHistory = async (): Promise<string | null> => {
  const results = await $history.loadHistoryChunk(0, CHUNK_SIZE);
  if (results.length === 0) return null;

  const resultIds = new Set(results.map((r) => r.id));
  const existingById = new Map(history.value.map((item) => [item.id, item]));

  const newRows = results.filter((r) => !existingById.has(r.id));
  const newItems = await Promise.all(newRows.map((raw) => processItem(raw)));

  // Known rows in database order - re-copied items jump back to the top.
  const knownOrdered = results
    .map((r) => existingById.get(r.id))
    .filter((item): item is HistoryItem => Boolean(item));

  const remaining = history.value.filter((item) => !resultIds.has(item.id));

  if (newItems.length > 0) {
    history.value = [...newItems, ...knownOrdered, ...remaining];
    return results[0].id;
  }

  const firstBefore = history.value[0]?.id;
  const firstAfter = knownOrdered[0]?.id;
  if (knownOrdered.length > 0 && firstBefore && firstAfter && firstBefore !== firstAfter) {
    // An existing item was re-copied and moved to the top. Reorder without
    // disturbing the selection (tracked by id).
    history.value = [...knownOrdered, ...remaining];
  }

  return null;
};

// Paste is guarded by a module-level, time-based lock (lib/appState.ts):
// no matter how many key handlers fire per Enter - stale or fresh - only
// the first one within the window can start a write+paste cycle.
const pasteSelectedItem = async (): Promise<void> => {
  const item = selectedItem.value;
  if (!item) return;
  if (!beginPaste()) return;

  try {
    let content = item.content;
    let contentType: string = item.content_type;
    if (contentType === "image") {
      try {
        content = await $history.readImage({ filename: content });
      } catch (error) {
        console.error("Error reading image file:", error);
        showToast("Could not read image");
        return;
      }
    }

    showActionsMenu.value = false;
    await hideApp();
    await $history.writeAndPaste({ content, contentType });
  } catch (error) {
    console.error("Paste failed:", error);
    showToast("Paste failed - try again");
    // Bring the app back so the user is not left stranded.
    try {
      await app.show();
      await window.getCurrentWindow().show();
      await window.getCurrentWindow().setFocus();
    } catch {
      // window may already be visible
    }
  }
};

const copyToClipboardOnly = async (item: HistoryItem): Promise<void> => {
  let content = item.content;
  if (item.content_type === ContentType.Image) {
    try {
      content = await $history.readImage({ filename: content });
    } catch (error) {
      console.error("Error reading image file:", error);
      showToast("Could not read image");
      return;
    }
  }
  try {
    await $history.writeClipboard({
      content,
      contentType: item.content_type,
    });
    showToast("Copied to clipboard");
  } catch (error) {
    console.error("Copy failed:", error);
    showToast("Copy failed");
  }
};

const togglePin = async (item: HistoryItem): Promise<void> => {
  try {
    const pinned = await $history.togglePin(item.id);
    item.pinned = pinned;
    showToast(pinned ? "Pinned to top" : "Unpinned");
  } catch (error) {
    console.error("Pin failed:", error);
    showToast("Pin failed");
  }
};

const deleteItem = async (item: HistoryItem): Promise<void> => {
  const neighbor = neighborOf(item);
  const wasSelected = selectedItemId.value === item.id;
  try {
    await $history.deleteHistoryItem(item.id);
  } catch (error) {
    console.error("Delete failed:", error);
    showToast("Delete failed");
    return;
  }
  history.value = history.value.filter((i) => i.id !== item.id);
  if (wasSelected) {
    selectItem(neighbor?.id ?? null);
  }
  showToast("Deleted");
};

const isYoutubeWatchUrl = (url: string): boolean => {
  return (
    /^(https?:\/\/)?(www\.)?(youtube\.com|youtu\.be)\/watch\?v=[\w-]+/.test(
      url
    ) || /^(https?:\/\/)?(www\.)?youtu\.be\/[\w-]+/.test(url)
  );
};

const getYoutubeThumbnail = (url: string): string => {
  let videoId;
  if (url.includes("youtu.be")) {
    videoId = url.split("youtu.be/")[1];
  } else {
    videoId = url.match(/[?&]v=([^&]+)/)?.[1];
  }
  return videoId
    ? `https://img.youtube.com/vi/${videoId}/maxresdefault.jpg`
    : "";
};

const toggleActionsMenu = (): void => {
  if (!selectedItem.value) return;
  showActionsMenu.value = !showActionsMenu.value;
};

const onActionsMenuAction = (id: string): void => {
  const item = selectedItem.value;
  showActionsMenu.value = false;
  if (!item) return;

  if (id === "copy") void copyToClipboardOnly(item);
  else if (id === "pin") void togglePin(item);
  else if (id === "delete") void deleteItem(item);
};

// Right-click an entry -> custom context menu with color tags. The menu is
// clamped to the fixed 750x474 window so it never renders off-screen.
const onItemContextMenu = (item: HistoryItem, event: MouseEvent): void => {
  const menuWidth = 196;
  const menuHeight = 92;
  const x = Math.max(6, Math.min(event.clientX, 750 - menuWidth - 6));
  const y = Math.max(6, Math.min(event.clientY, 474 - menuHeight - 6));
  tagMenu.value = { item, x, y };
};

const onTagSelect = async (color: string | null): Promise<void> => {
  const state = tagMenu.value;
  tagMenu.value = null;
  if (!state) return;

  try {
    await $history.tagItem(state.item.id, color);
    state.item.tag_color = color ?? undefined;
  } catch (error) {
    console.error("Tag failed:", error);
    showToast("Tag failed");
  }
};

// Idempotent: armKeyboard() wipes every registered handler first, so calling
// this any number of times (mount, focus) always leaves exactly ONE set of
// handlers on the single shared keyboard instance.
const setupKeyboard = (): void => {
  armKeyboard();

  keyboard.listen([Key.DownArrow], () => {
    selectNext();
  }, { prevent: true });

  keyboard.listen([Key.UpArrow], () => {
    selectPrevious();
  }, { prevent: true });

  keyboard.listen([Key.Enter], () => {
    pasteSelectedItem();
  }, { prevent: true });

  keyboard.listen([Key.Escape], () => {
    if (showActionsMenu.value) {
      showActionsMenu.value = false;
    } else {
      hideApp();
    }
  }, { prevent: true });

  keyboard.listen([Key.Delete], () => {
    const item = selectedItem.value;
    if (item) deleteItem(item);
  }, { prevent: true, ignoreIfEditable: true });

  switch (os.value) {
    case "macos":
      keyboard.listen([Key.Meta, Key.K], () => {
        toggleActionsMenu();
      }, { prevent: true });
      break;

    case "linux":
    case "windows":
      keyboard.listen([Key.Control, Key.K], () => {
        toggleActionsMenu();
      }, { prevent: true });
      break;
  }
};

// Every Tauri event listener is unmounted again, otherwise each page
// remount (e.g. returning from Settings) stacks another copy of every
// handler - the source of the doubled down-arrow and double paste.
let unlistenClipboard: (() => void) | null = null;
let unlistenFocus: (() => void) | null = null;
let unlistenBlur: (() => void) | null = null;

const setupEventListeners = async (): Promise<void> => {
  unlistenClipboard = await listen("clipboard-content-updated", async () => {
    lastUpdateTime.value = Date.now();

    if (searchQuery.value.trim()) {
      // Keep the active filter instead of dumping raw items into the list.
      searchHistory(searchQuery.value);
      return;
    }

    const newTopId = await updateHistory();
    if (newTopId) {
      selectItem(newTopId);
      scrollToTop();
    }
    // No new item (e.g. the same content was re-copied): the selection stays
    // where it is - it is tracked by id, so nothing is reset.
  });

  unlistenFocus = await listen("tauri://focus", async () => {
    const currentTime = Date.now();

    // Always re-sync with the database when the window is focused: it is one
    // cheap query, and it recovers any item whose clipboard event was missed
    // (e.g. event-loop hiccup) so new copies always show up.
    if (!searchQuery.value.trim()) {
      const newTopId = await updateHistory();
      lastUpdateTime.value = currentTime;
      if (newTopId) {
        // A new item arrived while hidden - jump to it.
        selectItem(newTopId);
        scrollToTop();
      }
      // Otherwise keep the current selection and scroll position.
    }

    focusSearchInput();
    // Re-arm the keyboard on focus (idempotent - can never stack handlers).
    setupKeyboard();
  });

  unlistenBlur = await listen("tauri://blur", () => {
    searchInputBlur();
  });
};

const teardownEventListeners = (): void => {
  unlistenClipboard?.();
  unlistenFocus?.();
  unlistenBlur?.();
  unlistenClipboard = null;
  unlistenFocus = null;
  unlistenBlur = null;
};

const hideApp = async (): Promise<void> => {
  try {
    await app.hide();
    await window.getCurrentWindow().hide();
  } catch (error) {
    console.error("Error hiding app:", error);
  }
};

const focusSearchInput = (): void => {
  nextTick(() => {
    topBar.value?.searchInput?.focus();
  });
};

const searchInputBlur = (): void => {
  topBar.value?.searchInput?.blur();
};

const onImageError = (): void => {
  // image dimensions already set to "Error" by the processor
};

const onPreviewError = (item: HistoryItem): void => {
  // The data URL failed to render (e.g. unsupported variant) - drop it so
  // the row falls back to the generic file icon.
  delete imageUrls.value[item.id];
};

// Scroll when the selection moves (tracked by id, stable across list rebuilds).
watch(
  selectedItemId,
  () => {
    if (selectedItemId.value) scrollToSelectedItem();
  },
  { flush: "post" }
);

onMounted(async () => {
  try {
    os.value = platform();
    showTypeFilters.value =
      (await $settings.getSetting("show_type_filters")) !== "false";
    await loadHistoryChunk();
    selectFirst();

    resultsContainer.value
      ?.osInstance()
      ?.elements()
      ?.viewport?.addEventListener("scroll", handleScroll);

    setupKeyboard();
    await setupEventListeners();
  } catch (error) {
    console.error("Error during onMounted:", error);
  }
});

onUnmounted(() => {
  teardownEventListeners();
  // The shared keyboard instance is NOT stopped here: this page's handlers
  // are simply replaced when the next page mounts (each onMounted clears and
  // re-registers). Stopping here would race the incoming page's setup.
  if (toastTimer) clearTimeout(toastTimer);
});

const getFormattedDate = computed(() => {
  if (!selectedItem.value?.timestamp) return "";
  return new Intl.DateTimeFormat("en-US", {
    dateStyle: "medium",
    timeStyle: "medium",
  }).format(selectedItem.value.timestamp);
});

const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return "0 Bytes";
  const k = 1024;
  const sizes = ["Bytes", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
};

const fetchPageMeta = async (url: string) => {
  try {
    const [title, ogImage] = (await invoke("fetch_page_meta", { url })) as [
      string,
      string | null
    ];
    pageTitle.value = title;
    if (ogImage) {
      pageOgImage.value = ogImage;
    }
  } catch (error) {
    console.error("Error fetching page meta:", error);
    pageTitle.value = "Error loading title";
  }
};

watch(
  () => selectedItem.value,
  (newItem) => {
    if (newItem?.content_type === ContentType.Link) {
      pageTitle.value = "Loading...";
      pageOgImage.value = "";
      fetchPageMeta(newItem.content);
    } else {
      pageTitle.value = "";
      pageOgImage.value = "";
    }
  }
);

// --- Code highlighting -----------------------------------------------------

const LANGUAGE_ALIASES: Record<string, string> = {
  js: "javascript",
  ts: "typescript",
  py: "python",
  rs: "rust",
  golang: "go",
  "c++": "cpp",
  "c#": "csharp",
  sh: "bash",
  shell: "bash",
  yml: "yaml",
  md: "markdown",
  rb: "ruby",
  kt: "kotlin",
};

const normalizeLanguage = (language?: string | null): string | null => {
  if (!language) return null;
  const key = language.toLowerCase().trim();
  if (LANGUAGE_ALIASES[key]) return LANGUAGE_ALIASES[key];
  return hljs.getLanguage(key) ? key : null;
};

// Best-effort code detection for items stored as plain text.
const looksLikeCode = (content: string): boolean => {
  if (!content || content.length < 24) return false;
  const lines = content.split("\n");
  if (lines.length < 2) return false;

  let score = 0;
  if (/\{[\s\S]*\}/.test(content)) score += 2;
  if (/=>/.test(content)) score += 2;
  if (/::/.test(content)) score += 1;
  if (/;\s*$/m.test(content)) score += 2;
  if (/\b(def|func|function|fn|class|struct|enum|public|private|import|package|const|let|var|return|SELECT|INSERT|UPDATE|DELETE)\b/.test(content))
    score += 2;
  if (/^\s{2,}\S/m.test(content)) score += 1; // indentation
  if (/^#!/.test(content)) score += 3;
  if (/^</.test(content)) score += 1;

  return score >= 4;
};

// Word-boundary patterns: "npm ", "apt ", ... used to match inside words
// ("identifier" contains "fi", "adapt" contains "apt") and let bash steal
// JSON/YAML snippets.
const CODE_LANGUAGE_PATTERNS: Array<[string, RegExp]> = [
  ["rust", /\b(fn|mut|pub|impl|match|struct|enum)\b/],
  ["python", /\b(def|elif|lambda|self|print|return)\b/],
  ["typescript", /\b(interface|readonly|type)\b/],
  ["javascript", /\b(const|let|var|function|console|document|window)\b/],
  ["go", /\bfunc\b|\bpackage\b|:=/],
  ["java", /\b(public|private|void)\b|System\.out/],
  ["cpp", /std::|#include </],
  ["c", /#include|#define|\bint main\b/],
  ["csharp", /using System|namespace |Console\.Write/],
  ["bash", /#!\/bin|\b(sudo|apt|npm|git|echo|curl)\b/],
  ["sql", /\b(SELECT|FROM|WHERE|INSERT INTO|UPDATE|JOIN|CREATE TABLE)\b/],
  ["html", /<[a-z][\s\S]*>/i],
  ["css", /(^|\s)[.#][a-z-]+\s*\{/m],
  ["php", /<\?php|\$\w+/],
  ["ruby", /\b(def|end|puts|require)\b/],
  ["swift", /\b(func|guard|print)\b/],
  ["kotlin", /\b(fun|val)\b/],
  ["yaml", /^\s*[a-z0-9_-]+:/m],
  ["toml", /^\s*\[[a-z0-9_-]+\]/m],
  ["markdown", /(^#{1,6}\s|\*\*[^*]+\*\*)/m],
];

// Returns the best-guessed language plus how confident the match is (number
// of pattern hits). A score >= 2 is strong enough to auto-highlight a text
// item; >= 1 still tells the badge what language we bet on.
const guessLanguage = (content: string): { language: string; score: number } | null => {
  const trimmed = content.trim();
  // JSON wins outright - keyword patterns misfire on config files that
  // mention "npm"/"git" (they were being labeled bash).
  if (trimmed.startsWith("{") && trimmed.includes('":')) {
    return { language: "json", score: 5 };
  }
  if (trimmed.startsWith("[")) {
    const quotes = (trimmed.match(/"/g) ?? []).length;
    if (quotes >= 2) return { language: "json", score: 4 };
  }

  let best: string | null = null;
  let bestScore = 0;
  for (const [language, pattern] of CODE_LANGUAGE_PATTERNS) {
    const matches = content.match(new RegExp(pattern.source, pattern.flags + "g"));
    const score = matches ? matches.length : 0;
    if (score > bestScore) {
      bestScore = score;
      best = language;
    }
  }
  return bestScore >= 1 ? { language: best, score: bestScore } : null;
};

const highlightCache = new Map<string, string>();

// Should the preview attempt syntax highlighting? Code items always; text
// items when they look like code OR a language pattern matched convincingly
// (>= 2 hits) - deliberately generous so highlighting is actually visible.
const shouldHighlight = (item: { content_type: ContentType; content: string }): boolean => {
  if (item.content_type === ContentType.Code) return true;
  if (looksLikeCode(item.content)) return true;
  const guess = guessLanguage(item.content);
  return guess !== null && guess.score >= 2;
};

// Language label shown on the preview badge (detected or stored). A
// confident guess overrides the stored label - older entries may have been
// mis-detected at copy time (e.g. JSON saved as bash).
const previewLanguage = computed((): string => {
  const item = selectedItem.value;
  if (!item || !shouldHighlight(item)) return "";
  const guess = guessLanguage(item.content);
  if (guess && guess.score >= 3) return guess.language;
  if (item.content_type === ContentType.Code) {
    return normalizeLanguage(item.language) ?? "";
  }
  return guess ? guess.language : "";
});

const highlightedCode = computed((): string => {
  const item = selectedItem.value;
  if (!item || !shouldHighlight(item)) return "";

  let language: string | null = null;
  const guess = guessLanguage(item.content);
  if (item.content_type === ContentType.Code) {
    language = normalizeLanguage(item.language);
    if (guess && guess.score >= 3) language = guess.language;
  } else if (guess && guess.score >= 2) {
    language = guess.language;
  }

  // Unknown/unregistered label -> let highlightAuto decide below.
  if (language && !hljs.getLanguage(language)) language = null;
  // No language at all and the text is too long to auto-detect cheaply.
  if (!language && item.content.length >= 8000) return "";

  const cached = highlightCache.get(item.id);
  if (cached !== undefined) return cached;

  let html = "";
  try {
    if (hljs.getLanguage(language)) {
      html = hljs.highlight(item.content, {
        language,
        ignoreIllegals: true,
      }).value;
    } else if (item.content.length < 8000) {
      // Unknown language label - let highlight.js pick the best match from
      // every registered language.
      html = hljs.highlightAuto(item.content).value;
    }
  } catch (error) {
    console.error("Highlight failed:", error);
    html = "";
  }
  highlightCache.set(item.id, html);
  return html;
});

// --- Info panel -------------------------------------------------------------

const getInfo = computed(() => {
  if (!selectedItem.value) return null;

  const baseInfo = {
    source: selectedItem.value.source,
    copied: selectedItem.value.timestamp,
  };

  const infoMap: Record<
    ContentType,
    () => InfoText | InfoImage | InfoFile | InfoLink | InfoColor | InfoCode
  > = {
    [ContentType.Text]: () => ({
      ...baseInfo,
      content_type: ContentType.Text,
      characters: selectedItem.value!.content.length,
      words: selectedItem.value!.content.trim().split(/\s+/).length,
    }),
    [ContentType.Image]: () => ({
      ...baseInfo,
      content_type: ContentType.Image,
      dimensions: imageDimensions.value[selectedItem.value!.id] || "Loading...",
      size: parseInt(imageSizes.value[selectedItem.value!.id] || "0"),
    }),
    [ContentType.File]: () => ({
      ...baseInfo,
      content_type: ContentType.File,
      path: selectedItem.value!.content,
      filesize: 0,
    }),
    [ContentType.Link]: () => ({
      ...baseInfo,
      content_type: ContentType.Link,
      title: pageTitle.value,
      url: selectedItem.value!.content,
      characters: selectedItem.value!.content.length,
    }),
    [ContentType.Color]: () => {
      const hex = selectedItem.value!.content;
      const r = parseInt(hex.slice(1, 3), 16);
      const g = parseInt(hex.slice(3, 5), 16);
      const b = parseInt(hex.slice(5, 7), 16);

      const rNorm = r / 255;
      const gNorm = g / 255;
      const bNorm = b / 255;

      const max = Math.max(rNorm, gNorm, bNorm);
      const min = Math.min(rNorm, gNorm, bNorm);
      let h = 0,
        s = 0;
      const l = (max + min) / 2;

      if (max !== min) {
        const d = max - min;
        s = l > 0.5 ? d / (2 - max - min) : d / (max + min);

        switch (max) {
          case rNorm:
            h = (gNorm - bNorm) / d + (gNorm < bNorm ? 6 : 0);
            break;
          case gNorm:
            h = (bNorm - rNorm) / d + 2;
            break;
          case bNorm:
            h = (rNorm - gNorm) / d + 4;
            break;
        }
        h /= 6;
      }

      return {
        ...baseInfo,
        content_type: ContentType.Color,
        hex: hex,
        rgb: `rgb(${r}, ${g}, ${b})`,
        hsl: `hsl(${Math.round(h * 360)}, ${Math.round(s * 100)}%, ${Math.round(
          l * 100
        )}%)`,
      };
    },
    [ContentType.Code]: () => ({
      ...baseInfo,
      content_type: ContentType.Code,
      language: selectedItem.value!.language ?? "Unknown",
      lines: selectedItem.value!.content.split("\n").length,
    }),
  };

  return infoMap[selectedItem.value.content_type]();
});

const infoRows = computed(() => {
  if (!getInfo.value) return [];

  const commonRows = [
    {
      label: "Source",
      value: getInfo.value.source,
      isUrl: false,
      icon: selectedItem.value?.source_icon
        ? `data:image/png;base64,${selectedItem.value.source_icon}`
        : undefined,
    },
    {
      label: "Content Type",
      value:
        getInfo.value.content_type.charAt(0).toUpperCase() +
        getInfo.value.content_type.slice(1),
      isUrl: false,
    },
  ];

  const typeSpecificRows: Record<
    ContentType,
    Array<{ label: string; value: string | number; isUrl?: boolean; icon?: string }>
  > = {
    [ContentType.Text]: [
      { label: "Characters", value: (getInfo.value as InfoText).characters },
      { label: "Words", value: (getInfo.value as InfoText).words },
    ],
    [ContentType.Image]: [
      { label: "Dimensions", value: (getInfo.value as InfoImage).dimensions },
      {
        label: "Image size",
        value: formatFileSize((getInfo.value as InfoImage).size),
      },
    ],
    [ContentType.File]: [
      { label: "Path", value: (getInfo.value as InfoFile).path },
    ],
    [ContentType.Link]: [
      ...((getInfo.value as InfoLink).title &&
        (getInfo.value as InfoLink).title !== "Loading..."
        ? [{ label: "Title", value: (getInfo.value as InfoLink).title || "" }]
        : []),
      { label: "URL", value: (getInfo.value as InfoLink).url, isUrl: true },
      { label: "Characters", value: (getInfo.value as InfoLink).characters },
    ],
    [ContentType.Color]: [
      { label: "Hex", value: (getInfo.value as InfoColor).hex },
      { label: "RGB", value: (getInfo.value as InfoColor).rgb },
      { label: "HSL", value: (getInfo.value as InfoColor).hsl },
    ],
    [ContentType.Code]: [
      { label: "Language", value: (getInfo.value as InfoCode).language },
      { label: "Lines", value: (getInfo.value as InfoCode).lines },
    ],
  };

  const specificRows = typeSpecificRows[getInfo.value.content_type].filter(
    (row) => row.value !== ""
  );

  return [
    ...commonRows,
    ...specificRows,
    { label: "Copied", value: getFormattedDate.value },
  ];
});
</script>

<style scoped lang="scss">
@use "/styles/index.scss";
</style>
