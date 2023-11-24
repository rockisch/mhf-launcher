<script setup>
import { onSettingsButton } from "../store";
import SettingsList from "../SettingsList.vue";

const emit = defineEmits(["back"]);
</script>

<template>
  <div class="flex w-full h-full text-white overflow-clip">
    <div
      class="flex flex-col shadow shadow-black p-3 bg-black/50 flex gap-2 h-full"
    >
      <div class="grow">
        <div class="btn btn-sm btn-primary">
          {{ $t("settings-general-title") }}
        </div>
      </div>
      <div class="btn btn-sm btn-primary w-max" @click="onSettingsButton">
        {{ $t("go-back-button") }}
      </div>
    </div>
    <div class="mhf-card !p-4 !pr-2 m-3 w-full overflow-clip">
      <SettingsList></SettingsList>
    </div>
  </div>
</template>
