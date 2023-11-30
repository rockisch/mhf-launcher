<script setup>
import { onSettingsButton } from "../store";
import SettingsList from "../settings/SettingsList.vue";

const emit = defineEmits(["back"]);
</script>

<template>
  <div class="flex w-full h-full text-white overflow-clip">
    <div
      class="flex flex-col shadow shadow-black p-3 bg-black/50 flex gap-2 h-full"
    >
      <a href="#general-settings" class="btn btn-sm px-6 btn-primary">
        {{ $t("settings-general-title") }}
      </a>
      <a href="#game-settings" class="btn btn-sm px-6 btn-primary">
        {{ $t("settings-game-title") }}
      </a>
      <a href="#advanced-settings" class="btn btn-sm px-6 btn-primary">
        {{ $t("settings-advanced-title") }}
      </a>
      <div class="grow"></div>
      <div class="btn btn-sm px-6 btn-primary w-max" @click="onSettingsButton">
        {{ $t("go-back-button") }}
      </div>
    </div>
    <div class="mhf-card !p-4 !pr-2 m-3 w-full overflow-clip">
      <SettingsList></SettingsList>
    </div>
  </div>
</template>
