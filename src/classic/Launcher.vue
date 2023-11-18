<script setup>
import "./style.css";

import { onMounted, onUnmounted, ref } from "vue";
import { computed } from "@vue/reactivity";
import { appWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/api/shell";

import Login from "./Login.vue";
import Characters from "./Characters.vue";
import MessageList from "./MessageList.vue";
import Settings from "./Settings.vue";
import { availableLocales } from "../fluent";
import {
  DELETE_DIALOG,
  SERVERS_DIALOG,
  LOGIN_PAGE,
  SETTINGS_PAGE,
  CHARACTERS_PAGE,
  openPicker,
  PATCHER_PAGE,
  PATCHER_DIALOG,
} from "../common";
import {
  store,
  storeMut,
  setCurrentEndpoint,
  closeDialog,
  effectiveFolder,
  dialogRemoveEndpoint,
  dialogSaveEndpoint,
  dialogDeleteCharacterConfirm,
  recentLog,
  currentBanner,
  bannerIndex,
  setBannerIndex,
  onSettingsButton,
  dialogCallback,
  patcherLog,
} from "../store";
import Patcher from "./Patcher.vue";

setCurrentEndpoint(store.currentEndpoint);

const localePicker = ref(false);
function openLocalePicker() {
  openPicker(localePicker);
}

const messages = computed(() => {
  let announcements = [];
  let news = [];
  for (const message of store.messages) {
    switch (message.kind) {
      case 1:
        announcements.push(message);
        break;
      default:
        news.push(message);
        break;
    }
  }
  return {
    announcements,
    news,
  };
});
</script>

<template>
  <div class="h-full w-full flex flex-col" :class="storeMut.locale">
    <div class="grow w-full h-0 flex text-white gap-8">
      <div class="flex flex-col items-center mb-2 mt-5">
        <div class="self-start">
          <img draggable="false" src="/classic/launcher-header.png" />
          <div class="absolute">
            <div class="relative bottom-[48px] left-[330px] text-[#dcdcdc]">
              release ver. 2.000
            </div>
          </div>
        </div>
        <div class="ml-3 h-full w-full grow flex flex-col items-center">
          <Characters v-if="storeMut.page === CHARACTERS_PAGE"></Characters>
          <Settings v-else-if="storeMut.page === SETTINGS_PAGE"></Settings>
          <template v-else>
            <Login v-if="storeMut.page === LOGIN_PAGE"></Login>
            <Patcher v-else-if="storeMut.page === PATCHER_PAGE"></Patcher>
            <div
              class="grow bg-[#00000099] border-[1px] border-white/20 w-full rounded-sm m-2 p-[6px] text-[14px] leading-[14px] h-0 w-[426px] max-w-[426px]"
            >
              <div class="overflow-auto scrollbar h-full">
                <div v-for="log in store.log" style="overflow-anchor: none">
                  <div :class="log.level">{{ log.message }}</div>
                </div>
                <div v-if="patcherLog">
                  <div class="warning">{{ patcherLog }}</div>
                </div>
                <div style="overflow-anchor: auto; height: 1px"></div>
              </div>
            </div>
          </template>
        </div>
        <button
          class="font-main cursor-pointer h-[31px] w-[279px] text-center border-[#d1c0a544] text-[#d1c0a5] font-['Shippori Mincho'] border-[1px] rounded bg-[#00000099] hover:bg-[#1b1b1b99] text-lg ml-3 mt-3"
          @click="onSettingsButton"
        >
          <span v-if="storeMut.page !== SETTINGS_PAGE">
            {{ $t("settings-button") }}
          </span>
          <span v-else>
            {{ $t("go-back-button") }}
          </span>
        </button>
      </div>
      <div class="w-[532px] flex flex-col mr-[30px] mt-[30px] mb-3 gap-4">
        <div class="flex gap-2">
          <img
            class="rounded shadow shadow-black shadow-md cursor-pointer"
            :src="currentBanner?.src"
            draggable="false"
            @click="open(currentBanner?.link)"
          />
          <div class="flex flex-col justify-center gap-3">
            <button
              v-for="(_, i) in store.banners"
              class="w-[10px] h-[10px] rounded-lg hover:bg-[#888888]"
              :class="i === bannerIndex ? 'bg-[#888888]' : 'bg-[#444444]'"
              @click="setBannerIndex(i)"
            ></button>
          </div>
        </div>
        <div
          class="grid grid-cols-[auto_auto_45px] auto-rows-auto gap-x-6 gap-y-2 overflow-auto scrollbar leading-4"
        >
          <MessageList
            :messages="messages.announcements"
            :title="$t('announcements-label')"
            :important="true"
          ></MessageList>
          <MessageList
            :messages="messages.news"
            :title="$t('news-label')"
          ></MessageList>
        </div>
        <div class="grow flex gap-8 flex-row-reverse">
          <div
            v-for="link in store.links"
            class="cursor-pointer text-[#9DA7B9] hover:text-[#C4C6CA] link-item"
            @click="open(link.link)"
          >
            <div
              class="rounded-[100px] h-[54px] w-[54px] mb-1 flex link-icon m-auto"
            >
              <img
                class="h-[32px] w-[32px] object-contain m-auto"
                draggable="false"
                :src="link.icon || '/classic/icon-inquiry.png'"
              />
            </div>
            <div class="text-sm text-center">{{ link.name }}</div>
          </div>
        </div>
      </div>
    </div>
    <div
      class="bg-[#00000080] h-[39px] col-span-2 flex gap-3 px-[30px] items-center overflow-clip"
    >
      <img src="/classic/capcom.png" class="object-contain" draggable="false" />
      <img src="/classic/cog.png" class="object-contain" draggable="false" />
      <div class="text-[#a0a0a0] text-sm">
        ©CAPCOM CO., LTD. ALL RIGHTS RESERVED.
      </div>
      <div class="grow text-right">
        <span v-if="recentLog" :class="recentLog.level">
          {{ recentLog.message }}
        </span>
      </div>
    </div>
  </div>
  <div
    data-tauri-drag-region
    class="absolute top-0 left-0 right-0 px-2 pb-2 flex gap-1 text-white/60 justify-start"
  >
    <div data-tauri-drag-region class="grow"></div>
    <div>
      <div
        class="locale-picker flex flex-col rounded-b bg-[#00000099] w-max leading-5 text-sm uppercase cursor-pointer"
      >
        <div
          class="flex w-[60px] hover:bg-[#1b1b1b99]"
          @click="openLocalePicker"
        >
          <img
            class="w-[16px] ml-2"
            :src="`/flags/${storeMut.locale}.svg`"
            draggable="false"
          />
          <span class="ml-2">{{ storeMut.locale }}</span>
        </div>
        <template v-if="localePicker">
          <template v-for="l in availableLocales">
            <template v-if="l !== storeMut.locale">
              <div
                class="flex w-[60px] hover:bg-[#1b1b1b99]"
                @click="storeMut.locale = l"
              >
                <img
                  class="w-[16px] ml-2"
                  :src="`/flags/${l}.svg`"
                  draggable="false"
                />
                <span class="ml-2">{{ l }}</span>
              </div>
            </template>
          </template>
        </template>
      </div>
    </div>
    <img
      @click="appWindow.minimize"
      src="/classic/minimize.png"
      class="h-[20px] w-[50px] state-img"
      draggable="false"
    />
    <img
      @click="appWindow.close"
      src="/classic/close.png"
      class="h-[20px] w-[50px] state-img"
      draggable="false"
    />
  </div>
  <dialog
    :open="store.dialogOpen"
    @close="closeDialog"
    class="absolute top-0 h-full w-full bg-transparent z-[10]"
  >
    <div class="flex items-center h-full">
      <div
        class="bg-[url('/classic/dialog.jpg')] bg-contain flex flex-col items-center m-auto news-default gap-1 px-14"
        :class="
          store.dialogKind === DELETE_DIALOG
            ? 'w-[560px] h-[320px] pt-[90px]'
            : 'w-[700px] h-[400px] pt-[112px]'
        "
      >
        <template
          v-if="store.dialogKind === DELETE_DIALOG && store.deleteCharacter"
          class=""
        >
          <div class="text-xl">
            {{ $t("delete-character-label") }}
          </div>
          <div class="warning">
            {{
              $t("delete-character-confirmation", {
                character_name: store.deleteCharacter.name,
              })
            }}
          </div>
        </template>
        <template v-else-if="store.dialogKind === PATCHER_DIALOG">
          <div class="text-xl">
            {{ $t("patcher-updates-label") }}
          </div>
          <div v-html="$t('patcher-updates-confirmation')"></div>
        </template>
        <template
          v-if="store.dialogKind === SERVERS_DIALOG && storeMut.editEndpoint"
        >
          <div class="text-xl">
            <span v-if="store.editEndpointNew">
              {{ $t("server-add-label") }}
            </span>
            <span v-else>
              {{ $t("server-edit-label") }}
            </span>
          </div>
          <div class="grid grid-cols-7 gap-x-2 items-end gap-y-0.5 px-[100px]">
            <label for="server-name" class="col-span-7">
              {{ $t("server-name-label") }}
            </label>
            <input
              v-model="storeMut.editEndpoint.name"
              type="text"
              class="box-text w-full col-span-5 text-white"
              spellcheck="false"
              :class="
                (store.editEndpointNew || storeMut.editEndpoint.isRemote
                  ? 'col-span-7'
                  : 'col-span-5') +
                (storeMut.editEndpoint.isRemote ? ' disabled' : '')
              "
              :disabled="storeMut.editEndpoint.isRemote"
            />
            <button
              v-if="!store.editEndpointNew && !storeMut.editEndpoint.isRemote"
              class="box-text box-btn col-span-2"
              @click.prevent="dialogRemoveEndpoint"
            >
              ❌ {{ $t("delete-button") }}
            </button>
            <label for="server-host" class="col-span-3">{{
              $t("server-host-label")
            }}</label>
            <label class="text-md news-default col-span-2">{{
              $t("server-launcher-port-label")
            }}</label>
            <label class="text-md news-default col-span-2">{{
              $t("server-game-port-label")
            }}</label>
            <input
              v-model="storeMut.editEndpoint.host"
              type="text"
              spellcheck="false"
              class="box-text w-full col-span-3 text-white"
              :class="{ disabled: storeMut.editEndpoint.isRemote }"
              :disabled="storeMut.editEndpoint.isRemote"
            />
            <input
              v-model.number="storeMut.editEndpoint.launcherPort"
              type="text"
              class="box-text col-span-2 text-white"
              spellcheck="false"
              placeholder="8080"
              :class="{ disabled: storeMut.editEndpoint.isRemote }"
              :disabled="storeMut.editEndpoint.isRemote"
            />
            <input
              v-model.number="storeMut.editEndpoint.gamePort"
              type="text"
              class="box-text col-span-2 text-white"
              spellcheck="false"
              placeholder="53310"
              :class="{ disabled: storeMut.editEndpoint.isRemote }"
              :disabled="storeMut.editEndpoint.isRemote"
            />
            <label class="text-md news-default col-span-7">{{
              $t("server-game-folder-label")
            }}</label>
            <input
              v-model="storeMut.editEndpoint.gameFolder"
              type="text"
              class="box-text col-span-7 text-white"
              spellcheck="false"
              :placeholder="effectiveFolder"
            />
          </div>
        </template>
        <div class="grow"></div>
        <div class="flex gap-12 m-4 news-default items-center justify-between">
          <form method="dialog">
            <button class="box-text box-lg box-btn">
              {{ $t("cancel-button") }}
            </button>
          </form>
          <div class="warning">
            {{ store.dialogError }}
          </div>
          <form method="dialog">
            <button
              class="box-text box-lg box-btn"
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
