<script setup>
import { open } from "@tauri-apps/api/shell";

import Login from "./Login.vue";
import Characters from "./Characters.vue";
import Settings from "./Settings.vue";
import Patcher from "./Patcher.vue";
import {
  storeMut,
  store,
  recentLog,
  bannerIndex,
  setBannerIndex,
  currentBanner,
  onSettingsButton,
  closeDialog,
  dismissRecentLog,
  dialogRemoveEndpoint,
  dialogCallback,
} from "../store";
import {
  LOGIN_PAGE,
  CHARACTERS_PAGE,
  SETTINGS_PAGE,
  formatDate,
  DELETE_DIALOG,
  SERVERS_DIALOG,
  PATCHER_DIALOG,
  PATCHER_PAGE,
} from "../common";
import { effectiveFolder } from "../store";

const alertClass = {
  info: "alert-info",
  warning: "alert-warning",
  error: "alert-error",
};
</script>

<template>
  <div class="w-full h-full">
    <Settings
      v-if="storeMut.page == SETTINGS_PAGE"
      class="w-full h-full"
      @back="storeMut.page = prevPage"
    >
    </Settings>
    <div
      v-else
      class="grid p-3 gap-3 grid-cols-[515px_auto] grid-rows-[135px_auto] w-full h-full"
    >
      <div>
        <img
          id="banner"
          class="rounded shadow shadow-black cursor-pointer"
          draggable="false"
          :src="currentBanner?.src"
          @click="open(currentBanner?.link)"
        />
        <div class="flex gap-2 justify-center">
          <button
            v-for="(_, i) in store.banners"
            class="w-[8px] h-[8px] rounded-lg hover:bg-[#888888] my-2"
            :class="i === bannerIndex ? 'bg-[#888888]' : 'bg-[#444444]'"
            @click="setBannerIndex(i)"
          ></button>
        </div>
      </div>
      <div class="flex flex-col gap-3 row-span-2 overflow-hidden">
        <Login v-if="storeMut.page == LOGIN_PAGE"></Login>
        <Characters v-else-if="storeMut.page == CHARACTERS_PAGE"></Characters>
        <Patcher v-else-if="storeMut.page == PATCHER_PAGE"></Patcher>
        <div
          class="grow flex gap-2 flex-row-reverse content-start flex-wrap-reverse"
        >
          <button
            class="btn btn-sm btn-primary"
            @click="onSettingsButton"
            :disabled="store.authLoading"
          >
            {{ $t("settings-button") }}
          </button>
          <button
            v-for="link in store.links"
            class="btn btn-sm btn-ghost text-[#A6D8FF]"
            @click="open(link.link)"
          >
            <img
              v-if="link.icon"
              :src="link.icon"
              class="h-[14px] link-image"
              draggable="false"
            />
            {{ link.name }}
          </button>
        </div>
      </div>
      <div
        class="grid gap-0.5 gap-x-2 grid-cols-[max-content_auto] overflow-auto content-start ml-1 scrollbar"
      >
        <template v-for="message in store.messages">
          <span class="py-1" :class="{ 'text-yellow-300': message.kind == 1 }">
            {{ formatDate(message.date) }}
          </span>
          <button
            class="btn btn-sm btn-ghost mr-2 h-max px-1.5 py-1 text-start justify-start leading-5 text-[16px]"
            :class="{ 'text-yellow-300': message.kind == 1 }"
            @click="open(message.link)"
          >
            {{ message.message }}
          </button>
        </template>
      </div>
    </div>
  </div>
  <div
    v-if="recentLog"
    class="toast toast-start z-[5]"
    @click="dismissRecentLog"
  >
    <div class="alert cursor-pointer py-2" :class="alertClass[recentLog.level]">
      <span class="text-black font-medium">{{ recentLog.message }}</span>
    </div>
  </div>
  <dialog
    :open="store.dialogOpen"
    @close="closeDialog"
    class="absolute top-0 h-full w-full bg-transparent z-[10]"
  >
    <div
      v-if="store.dialogOpen"
      class="flex items-center justify-center h-full"
    >
      <div class="absolute top-0 left-0 h-full w-full bg-black/25"></div>
      <div
        class="modal-box rounded-lg shadow shadow-black bg-[#111] text-white flex flex-col gap-4"
      >
        <template v-if="store.dialogKind === DELETE_DIALOG">
          <h3 class="font-bold text-lg">{{ $t("delete-character-label") }}</h3>
          <p class="py-4">
            {{
              $t("delete-character-confirmation", {
                character_name: store.deleteCharacter.name,
              })
            }}
          </p>
        </template>
        <template v-else-if="store.dialogKind === PATCHER_DIALOG">
          <h3 class="font-bold text-lg">{{ $t("patcher-updates-label") }}</h3>
          <p class="py-4" v-html="$t('patcher-updates-confirmation')"></p>
        </template>
        <template v-else-if="store.dialogKind === SERVERS_DIALOG">
          <h3 class="font-bold text-lg">
            <span v-if="store.editEndpointNew">
              {{ $t("server-add-label") }}
            </span>
            <span v-else>
              {{ $t("server-edit-label") }}
            </span>
          </h3>
          <div class="grid grid-cols-7 gap-y-0.5 gap-x-3">
            <label for="server-name" class="col-span-7 mt-1">
              {{ $t("server-name-label") }}
            </label>
            <input
              v-model="storeMut.editEndpoint.name"
              type="text"
              spellcheck="false"
              class="input input-sm input-primary"
              :class="
                store.editEndpointNew || storeMut.editEndpoint.isRemote
                  ? 'col-span-7'
                  : 'col-span-5'
              "
              :disabled="storeMut.editEndpoint.isRemote"
            />
            <button
              v-if="!store.editEndpointNew && !storeMut.editEndpoint.isRemote"
              class="btn btn-sm btn-primary col-span-2"
              @click.prevent="dialogRemoveEndpoint"
            >
              ❌ {{ $t("delete-button") }}
            </button>
            <label for="server-host" class="col-span-3 mt-1">
              {{ $t("server-host-label") }}
            </label>
            <label class="col-span-2 mt-1">
              {{ $t("server-launcher-port-label") }}
            </label>
            <label class="col-span-2 mt-1">
              {{ $t("server-game-port-label") }}
            </label>
            <input
              v-model="storeMut.editEndpoint.host"
              type="text"
              spellcheck="false"
              class="input input-sm input-primary col-span-3"
              :disabled="storeMut.editEndpoint.isRemote"
            />
            <input
              v-model.number="storeMut.editEndpoint.launcherPort"
              type="text"
              class="input input-sm input-primary col-span-2"
              spellcheck="false"
              placeholder="8080"
              :disabled="storeMut.editEndpoint.isRemote"
            />
            <input
              v-model.number="storeMut.editEndpoint.gamePort"
              type="text"
              class="input input-sm input-primary col-span-2"
              spellcheck="false"
              placeholder="53310"
              :disabled="storeMut.editEndpoint.isRemote"
            />
            <label class="col-span-7 mt-1">
              {{ $t("server-game-folder-label") }}
            </label>
            <input
              v-model="storeMut.editEndpoint.gameFolder"
              type="text"
              class="input input-sm input-primary col-span-7"
              spellcheck="false"
              :placeholder="effectiveFolder"
            />
          </div>
        </template>
        <div class="flex justify-between gap-2 items-center">
          <form method="dialog">
            <button class="btn btn-sm btn-primary">
              {{ $t("cancel-button") }}
            </button>
          </form>
          <div class="warning">
            {{ store.dialogError }}
          </div>
          <form method="dialog">
            <button
              class="btn btn-sm btn-primary"
              @click.prevent="dialogCallback"
            >
              <span v-if="store.dialogKind === DELETE_DIALOG">
                {{ $t("delete-button") }}
              </span>
              <span v-else-if="store.dialogKind === PATCHER_DIALOG">
                {{ $t("install-button") }}
              </span>
              <span v-else-if="store.editEndpointNew">
                {{ $t("add-button") }}
              </span>
              <span v-else>
                {{ $t("save-button") }}
              </span>
            </button>
          </form>
        </div>
      </div>
    </div>
  </dialog>
</template>
