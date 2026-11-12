<template>
  <div>
    <Noise />
    <NuxtPage />
  </div>
</template>

<script setup lang="ts">
import { listen } from "@tauri-apps/api/event";
import { app, window } from "@tauri-apps/api";
import { disable, enable } from "@tauri-apps/plugin-autostart";
import { onMounted } from "vue";

const { $settings } = useNuxtApp();

onMounted(async () => {
  await listen("settings", async () => {
    await navigateTo("/settings");
    await app.show();
    await window.getCurrentWindow().show();
  });

  if ((await $settings.getSetting("autostart")) === "true") {
    await enable();
  } else {
    await disable();
  }

  await listen("main_route", async () => {
    await navigateTo("/");
  });
});
</script>

<style lang="scss">
@font-face {
  font-family: SFRoundedRegular;
  font-display: swap;
  src: url("/fonts/SFRoundedRegular.otf") format("opentype");
}

@font-face {
  font-family: SFRoundedMedium;
  font-display: swap;
  src: url("/fonts/SFRoundedMedium.otf") format("opentype");
}

@font-face {
  font-family: SFRoundedSemiBold;
  font-display: swap;
  src: url("/fonts/SFRoundedSemiBold.otf") format("opentype");
}

@font-face {
  font-family: CommitMono;
  font-display: swap;
  src: url("/fonts/CommitMono.woff2") format("woff2");
}

// Default theme: Ember (also used when no data-theme attribute is set)
:root {
  --background: #2e2d2b;
  --surface: rgba(46, 45, 43, 0.051);
  --accent: #feb453;
  --border: #ffffff0d;
  --selection: #ffffff12;

  --text: #e5dfd5;
  --text-secondary: #ada9a1;
  --text-muted: #78756f;
  --icon: #e5dfd5;

  --code-keyword: #feb453;
  --code-string: #a8cc8c;
  --code-comment: #78756f;
  --code-number: #d3869b;
  --code-function: #90c1d8;
  --code-type: #e6c07b;
  --code-variable: #e5dfd5;
  --code-attr: #a8cc8c;
  --code-meta: #c68f51;

  --sidebar-width: 286px;
  --bottom-bar-height: 39px;
  --info-panel-height: 160px;
  --content-view-height: calc(
    100% - var(--search-height) - var(--info-panel-height) -
      var(--bottom-bar-height)
  );

  color-scheme: dark;
}

:root[data-theme="midnight"] {
  --background: #171c28;
  --surface: rgba(23, 28, 40, 0.051);
  --accent: #7aa2f7;
  --border: #ffffff10;
  --selection: #7aa2f726;

  --text: #dce3f2;
  --text-secondary: #93a0b8;
  --text-muted: #5d6b85;
  --icon: #dce3f2;

  --code-keyword: #7aa2f7;
  --code-string: #9ece6a;
  --code-comment: #5d6b85;
  --code-number: #f7768e;
  --code-function: #7dcfff;
  --code-type: #e0af68;
  --code-variable: #dce3f2;
  --code-attr: #9ece6a;
  --code-meta: #bb9af7;
}

:root[data-theme="forest"] {
  --background: #212921;
  --surface: rgba(33, 41, 33, 0.051);
  --accent: #9bd47c;
  --border: #ffffff0d;
  --selection: #9bd47c26;

  --text: #e2e9df;
  --text-secondary: #a3b09e;
  --text-muted: #6f7d6c;
  --icon: #e2e9df;

  --code-keyword: #9bd47c;
  --code-string: #d8c686;
  --code-comment: #6f7d6c;
  --code-number: #d39a9b;
  --code-function: #8fc7d8;
  --code-type: #c9b458;
  --code-variable: #e2e9df;
  --code-attr: #d8c686;
  --code-meta: #b08968;
}

:root[data-theme="crimson"] {
  --background: #2b2124;
  --surface: rgba(43, 33, 36, 0.051);
  --accent: #f2889c;
  --border: #ffffff0d;
  --selection: #f2889c26;

  --text: #ecdfdc;
  --text-secondary: #b09e9c;
  --text-muted: #7d6f6d;
  --icon: #ecdfdc;

  --code-keyword: #f2889c;
  --code-string: #c9c386;
  --code-comment: #7d6f6d;
  --code-number: #d8a378;
  --code-function: #a3c2d8;
  --code-type: #e0b458;
  --code-variable: #ecdfdc;
  --code-attr: #c9c386;
  --code-meta: #b08968;
}

:root[data-theme="paper"] {
  --background: #f3efe7;
  --surface: rgba(243, 239, 231, 0.6);
  --accent: #b97a24;
  --border: #00000014;
  --selection: #00000010;

  --text: #37332e;
  --text-secondary: #6f6a62;
  --text-muted: #9a948a;
  --icon: #4a4640;

  --code-keyword: #b97a24;
  --code-string: #4e7a46;
  --code-comment: #9a948a;
  --code-number: #a34d6a;
  --code-function: #3d6f8a;
  --code-type: #8a6d1d;
  --code-variable: #37332e;
  --code-attr: #4e7a46;
  --code-meta: #8a5a2b;

  color-scheme: light;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
  text-decoration: none;
  font-family: SFRoundedRegular;
  scroll-behavior: smooth;
  scrollbar-width: thin;
  user-select: none;

  --os-handle-bg: var(--text-muted);
  --os-handle-bg-hover: var(--text-secondary);
  --os-handle-bg-active: var(--text-secondary);
}

// The window is a fixed 750x474 canvas: the page itself must NEVER scroll,
// otherwise scrolling past the edge of an inner list pushes the entire GUI
// up out of the window. Block page scroll and scroll-chaining outright.
html,
body {
  background-color: transparent;
  width: 750px;
  height: 474px;
  z-index: -1;
  font-size: 14px;
  overflow: hidden;
  overscroll-behavior: none;
}

// Every inner scroller must stop chaining to the page when it hits its edge.
.os-viewport,
pre.code-preview,
.code-preview code {
  overscroll-behavior: contain;
}

.os-scrollbar-horizontal {
  display: none;
}
</style>
