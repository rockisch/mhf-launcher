<script setup>
import { open } from "@tauri-apps/api/dialog";

import { CLASSIC_STYLE, MODERN_STYLE } from "../common";
import { effectiveFolder, storeMut } from "../store";

async function onChooseFolder() {
  const folder = await open({ directory: true });
  if (folder !== null) {
    storeMut.gameFolder = folder;
  }
}
</script>

<template>
  <div class="mt-2 box-text mx-2 !px-10 !py-2 flex flex-col gap-2">
    <div>
      <h2 class="text-xl pb-1">
        {{ $t("style-label") }}
      </h2>
      <select v-model="storeMut.style" name="style" class="box-text">
        <option :value="CLASSIC_STYLE">{{ $t("classic-style") }}</option>
        <option :value="MODERN_STYLE">{{ $t("modern-style") }}</option>
      </select>
    </div>
    <div>
      <h2 class="text-xl pb-1">
        {{ $t("game-folder-label") }}
      </h2>
      <label class="input-label cursor-pointer py-0.5">
        <input
          type="radio"
          class="cursor-pointer accent-black"
          name="game-folder"
          :checked="storeMut.gameFolder === null"
          @change="storeMut.gameFolder = null"
        />
        <span>
          {{ $t("current-folder-label") }}
        </span>
      </label>
      <label class="input-label cursor-pointer py-0.5">
        <input
          type="radio"
          class="cursor-pointer accent-black"
          name="game-folder"
          :checked="storeMut.gameFolder !== null"
          @change="storeMut.gameFolder = effectiveFolder"
        />
        <button
          class="box-text box-btn"
          :disabled="storeMut.gameFolder === null"
          @click="onChooseFolder"
        >
          {{ effectiveFolder }}
        </button>
      </label>
    </div>
    <div>
      <h2 class="text-xl pb-1">
        {{ $t("serverlist-url-label") }}
      </h2>
      <input
        v-model.lazy.trim="storeMut.serverlistUrl"
        class="box-text w-full"
        type="url"
        spellcheck="false"
        :placeholder="DEFAULT_SERVERLIST_URL"
      />
    </div>
  </div>
</template>
