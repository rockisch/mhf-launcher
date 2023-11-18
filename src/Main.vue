<script setup>
import { ref } from "vue";
import { listen, once } from "@tauri-apps/api/event";

import "./style.css";

import {
  storeMut,
  initStore,
  initRemoteEndpoints,
  updatePatcher,
  logText,
} from "./store";
import ClassicLauncher from "./classic/Launcher.vue";
import ModernLauncher from "./modern/Launcher.vue";
import { MODERN_STYLE, CLASSIC_STYLE } from "./common";

const initialLoaded = ref(false);

initStore().then(() => (initialLoaded.value = true));
listen("endpoints", ({ payload }) => {
  initRemoteEndpoints(payload);
});
listen("messages", ({ payload }) => {});
listen("patcher", ({ payload }) => {
  updatePatcher(payload);
});
listen("log", ({ payload }) => {
  logText(payload.level, payload.message);
});
</script>

<template>
  <template v-if="initialLoaded">
    <ClassicLauncher v-if="storeMut.style == CLASSIC_STYLE"></ClassicLauncher>
    <ModernLauncher v-else-if="storeMut.style == MODERN_STYLE"></ModernLauncher>
  </template>
</template>
