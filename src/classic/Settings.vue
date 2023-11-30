<script setup>
import { open } from "@tauri-apps/api/dialog";

import { onSettingsButton, storeMut } from "../store";
import SettingsList from "../settings/SettingsList.vue";

async function onChooseFolder() {
  const folder = await open({ directory: true });
  if (folder !== null) {
    storeMut.gameFolder = folder;
  }
}
</script>

<template>
  <div class="flex gap-2 mx-2 grow mt-7 mb-2 overflow-clip">
    <div class="flex flex-col box-text gap-2 !py-2">
      <a href="#general-settings" class="box-text box-btn !px-8 text-center">
        {{ $t("settings-general-title") }}
      </a>
      <a href="#game-settings" class="box-text box-btn !px-8 text-center">
        {{ $t("settings-game-title") }}
      </a>
      <a href="#advanced-settings" class="box-text box-btn !px-8 text-center">
        {{ $t("settings-advanced-title") }}
      </a>
      <div class="grow"></div>
      <div class="box-text box-btn !px-8 text-center" @click="onSettingsButton">
        {{ $t("go-back-button") }}
      </div>
    </div>
    <div class="grow box-text !px-4 !py-4 flex flex-col gap-2 overflow-hidden">
      <SettingsList></SettingsList>
    </div>
  </div>
</template>
