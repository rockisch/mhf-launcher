<script setup>
import { ref } from "vue";
import { open } from "@tauri-apps/api/dialog";

import { CLASSIC_STYLE, MODERN_STYLE, DEFAULT_SERVERLIST_URL } from "../common";
import { onSettingsButton, storeMut, effectiveFolder } from "../store";

const emit = defineEmits(["back"]);

const editEndpoints = ref(null);
const editEndpointsRef = ref(null);

async function onChooseFolder() {
  const folder = await open({ directory: true });
  if (folder !== null) {
    storeMut.gameFolder = folder;
  }
}
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
      <div class="overflow-auto h-full scrollbar pr-2">
        <h1 class="text-3xl mb-4">
          {{ $t("settings-general-title") }}
        </h1>
        <div class="flex flex-col gap-2">
          <h2 class="text-xl">
            {{ $t("style-label") }}
          </h2>
          <select
            v-model="storeMut.style"
            class="select select-primary select-sm w-max"
          >
            <option :value="CLASSIC_STYLE">{{ $t("classic-style") }}</option>
            <option :value="MODERN_STYLE">{{ $t("modern-style") }}</option>
          </select>
          <div class="divider my-0 py-0"></div>
          <h2 class="text-xl">
            {{ $t("game-folder-label") }}
          </h2>
          <label class="label cursor-pointer pb-1 pt-1">
            <input
              type="radio"
              class="radio radio-sm"
              name="game-folder"
              :checked="storeMut.gameFolder === null"
              @change="storeMut.gameFolder = null"
            />
            <span class="label-text">
              {{ $t("current-folder-label") }}
            </span>
          </label>
          <label class="label cursor-pointer pt-0">
            <input
              type="radio"
              class="radio radio-sm"
              name="game-folder"
              :checked="storeMut.gameFolder !== null"
              @change="storeMut.gameFolder = effectiveFolder"
            />
            <button
              class="btn btn-sm btn-primary w-max"
              :disabled="storeMut.gameFolder === null"
              @click="onChooseFolder"
            >
              {{ effectiveFolder }}
            </button>
          </label>
          <div class="divider my-0 py-0"></div>
          <h2 class="text-xl">
            {{ $t("list-remote-servers-label") }}
          </h2>
          <label class="label cursor-pointer">
            <input
              :checked="storeMut.serverlistUrl !== ''"
              @change="
                storeMut.serverlistUrl = $event.target.checked
                  ? DEFAULT_SERVERLIST_URL
                  : ''
              "
              type="checkbox"
              class="checkbox checkbox-info checkbox-sm"
            />
            <span class="label-text">
              {{ $t("enable-button") }}
            </span>
          </label>
          <template v-if="storeMut.serverlistUrl !== ''">
            <h2>
              {{ $t("serverlist-url-label") }}
            </h2>
            <input
              v-model.lazy.trim="storeMut.serverlistUrl"
              class="input input-sm input-primary"
              type="url"
              spellcheck="false"
            />
          </template>
        </div>
      </div>
    </div>
  </div>
</template>
