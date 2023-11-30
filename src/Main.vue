<script setup>
import { ref } from "vue";
import { listen, once } from "@tauri-apps/api/event";

import "./style.css";

import {
  storeMut,
  initStore,
  initRemoteEndpoints,
  updateRemoteMessages,
  updatePatcher,
  logText,
} from "./store";
import ClassicLauncher from "./classic/Launcher.vue";
import ModernLauncher from "./modern/Launcher.vue";
import { MODERN_STYLE, CLASSIC_STYLE } from "./common";
import { logMessage } from "./store";

const initialLoaded = ref(false);

initStore().then(() => (initialLoaded.value = true));
listen("userdata", ({ payload }) => {
  storeMut.username = payload.userdata.username;
  storeMut.password = payload.password;
  storeMut.rememberMe = payload.userdata.rememberMe;
});
listen("endpoints", ({ payload }) => {
  initRemoteEndpoints(payload);
});
listen("remote_messages", ({ payload }) => {
  updateRemoteMessages(payload);
});
listen("patcher", ({ payload }) => {
  updatePatcher(payload);
});
listen("log", ({ payload }) => {
  logMessage(payload.level, payload.message);
});
</script>

<template>
  <template v-if="initialLoaded">
    <ClassicLauncher v-if="storeMut.style == CLASSIC_STYLE"></ClassicLauncher>
    <ModernLauncher v-else-if="storeMut.style == MODERN_STYLE"></ModernLauncher>
  </template>
</template>
