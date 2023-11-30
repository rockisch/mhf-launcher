<script setup>
import { open } from "@tauri-apps/api/dialog";

import {
  CLASSIC_STYLE,
  MODERN_STYLE,
  DEFAULT_SERVERLIST_URL,
  DEFAULT_MESSAGELIST_URL,
} from "../common";
import { storeMut, effectiveFolder, store, setSetting } from "../store";
import SettingsItem from "./SettingsItem.vue";
import SettingsCheckbox from "./SettingsCheckbox.vue";

async function onChooseFolder() {
  const folder = await open({ directory: true });
  if (folder !== null) {
    storeMut.gameFolder = folder;
  }
}

function setNumber(name, event) {
  let value = event.target.value;
  if (value === "") {
    value = 0;
  } else {
    value = parseInt(value);
  }
  if (!isNaN(value) && value > 0) {
    setSetting(name, value);
  } else {
    event.target.value = store.settings[name];
  }
}
</script>

<template>
  <div class="overflow-auto h-full scrollbar pr-2 flex flex-col gap-3">
    <h1 id="general-settings" class="text-3xl">
      {{ $t("settings-general-title") }}
    </h1>
    <div class="flex flex-col gap-2">
      <SettingsItem :name="$t('style-label')">
        <select
          v-model="storeMut.style"
          class="select select-primary select-sm w-max"
        >
          <option :value="MODERN_STYLE">{{ $t("modern-style") }}</option>
          <option :value="CLASSIC_STYLE">{{ $t("classic-style") }}</option>
        </select>
      </SettingsItem>
      <SettingsItem :name="$t('game-folder-label')">
        <label class="label cursor-pointer">
          <input
            type="radio"
            name="game-folder"
            :checked="storeMut.gameFolder === null"
            @change="storeMut.gameFolder = null"
          />
          <span class="label-text">
            {{ $t("current-folder-label") }}
          </span>
        </label>
        <label class="label cursor-pointer">
          <input
            type="radio"
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
      </SettingsItem>
    </div>
    <div class="divider my-0 py-0"></div>
    <h1 id="game-settings" class="text-3xl">
      {{ $t("settings-game-title") }}
    </h1>
    <div class="flex flex-col gap-2">
      <SettingsCheckbox
        :model-value="store.settings.hdVersion"
        @update:model-value="setSetting('hdVersion', $event)"
        :name="$t('hd-version-label')"
      ></SettingsCheckbox>
      <SettingsCheckbox
        :model-value="store.settings.fullscreen"
        @update:model-value="setSetting('fullscreen', $event)"
        :name="$t('fullscreen-label')"
      ></SettingsCheckbox>
      <SettingsItem :name="$t('window-resolution-label')">
        <div class="flex gap-1">
          <input
            :value="store.settings.windowW"
            @change="setNumber('windowW', $event)"
            inputmode="numeric"
            pattern="[0-9]*"
            class="input input-sm input-primary w-[90px]"
          />
          x
          <input
            :value="store.settings.windowH"
            @change="setNumber('windowH', $event)"
            inputmode="numeric"
            pattern="[0-9]*"
            class="input input-sm input-primary w-[90px]"
          />
        </div>
      </SettingsItem>
      <SettingsItem :name="$t('fullscreen-resolution-label')">
        <div class="flex gap-1">
          <input
            :value="store.settings.fullscreenW"
            @change="setNumber('fullscreenW', $event)"
            inputmode="numeric"
            pattern="[0-9]*"
            class="input input-sm input-primary w-[90px]"
          />
          x
          <input
            :value="store.settings.fullscreenH"
            @change="setNumber('fullscreenH', $event)"
            inputmode="numeric"
            pattern="[0-9]*"
            class="input input-sm input-primary w-[90px]"
          />
        </div>
      </SettingsItem>
    </div>
    <div class="divider my-0 py-0"></div>
    <h1 id="advanced-settings" class="text-3xl">
      {{ $t("settings-advanced-title") }}
    </h1>
    <div class="flex flex-col gap-2">
      <SettingsCheckbox
        :name="$t('list-remote-servers-label')"
        :model-value="storeMut.serverlistUrl !== ''"
        @update:model-value="
          storeMut.serverlistUrl = $event ? DEFAULT_SERVERLIST_URL : ''
        "
      >
        <template v-if="storeMut.serverlistUrl !== ''" v-slot:extended>
          <input
            v-model.lazy.trim="storeMut.serverlistUrl"
            class="input input-sm input-primary"
            type="url"
            spellcheck="false"
          />
        </template>
      </SettingsCheckbox>
      <SettingsCheckbox
        :name="$t('list-remote-messages-label')"
        :model-value="storeMut.messagelistUrl !== ''"
        @update:model-value="
          storeMut.messagelistUrl = $event ? DEFAULT_MESSAGELIST_URL : ''
        "
      >
        <template v-if="storeMut.messagelistUrl !== ''" v-slot:extended>
          <input
            v-model.lazy.trim="storeMut.messagelistUrl"
            class="input input-sm input-primary"
            type="url"
            spellcheck="false"
          />
        </template>
      </SettingsCheckbox>
    </div>
  </div>
</template>
