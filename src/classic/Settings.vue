<script setup>
import { open } from "@tauri-apps/api/dialog";

import {
  CLASSIC_STYLE,
  DEFAULT_MESSAGELIST_URL,
  DEFAULT_SERVERLIST_URL,
  MODERN_STYLE,
} from "../common";
import { effectiveFolder, storeMut } from "../store";
import SettingsList from "../SettingsList.vue";

async function onChooseFolder() {
  const folder = await open({ directory: true });
  if (folder !== null) {
    storeMut.gameFolder = folder;
  }
}
</script>

<template>
  <div
    class="mt-2 box-text mx-2 !px-4 !py-4 flex flex-col gap-2 overflow-hidden"
  >
    <SettingsList style="width: 400px"></SettingsList>
  </div>
</template>
