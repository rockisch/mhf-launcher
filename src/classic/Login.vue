<script setup>
import { ref } from "vue";

import { openPicker } from "../common";
import {
  store,
  storeMut,
  setCurrentEndpoint,
  doLogin,
  doRegister,
  dialogEditEndpoint,
  dialogAddEndpoint,
} from "../store";

const serverPicker = ref(false);

// Needed because templates always deref
function openPickerRef() {
  openPicker(serverPicker);
}
</script>

<template>
  <div class="grow flex flex-col items-center w-full h-full mt-2 px-12">
    <div class="min-w-[250px] flex flex-col">
      <label for="username_input">{{ $t("username-label") }}</label>
      <input
        v-model="storeMut.username"
        type="text"
        id="username_input"
        class="box-text"
        spellcheck="false"
        :disabled="store.authLoading"
      />
    </div>
    <div class="min-w-[250px] flex flex-col">
      <label for="password_input">{{ $t("password-label") }}</label>
      <input
        v-model="storeMut.password"
        type="password"
        id="password_input"
        class="box-text"
        :disabled="store.authLoading"
      />
    </div>
    <div class="flex flex-col">
      <label>{{ $t("server-select-label") }}</label>
      <div class="h-[50x] min-w-[250px] z-[1]">
        <div
          class="box-text cursor-pointer flex items-center"
          :class="{ 'box-disabled': store.authLoading }"
          @click="store.authLoading ? null : openPickerRef()"
        >
          <div class="grow">
            <span>{{ store.currentEndpoint.name }}</span>
          </div>
          <div :class="serverPicker ? 'arrow-up' : 'arrow-down'"></div>
        </div>
        <div
          v-if="serverPicker"
          class="absolute z-[-1] rounded-b mt-[-1px] bg-[#000000f0] border-[1px] border-t-0 border-white/20 w-[250px] cursor-pointer pt-0.5 max-h-[250px] overflow-auto scrollbar"
        >
          <div
            v-if="store.remoteEndpoints"
            class="border-b-[1px] border-white/20"
          >
            <div
              v-for="(endpoint, i) in store.remoteEndpoints"
              class="text-sm flex"
            >
              <span
                class="py-0.5 px-2 grow hover:bg-[#304368b8]"
                @click="setCurrentEndpoint(endpoint)"
              >
                {{ endpoint.name }}
              </span>
            </div>
          </div>
          <div v-if="store.endpoints" class="border-b-[1px] border-white/20">
            <div v-for="(endpoint, i) in store.endpoints" class="text-sm flex">
              <span
                class="py-0.5 px-2 grow hover:bg-[#304368b8]"
                @click="setCurrentEndpoint(endpoint)"
              >
                {{ endpoint.name }}
              </span>
              <span
                class="py-0.5 px-1.5 hover:bg-[#304368b8]"
                @click="dialogEditEndpoint(i)"
              >
                ⚙
              </span>
            </div>
          </div>
          <div class="text-sm flex">
            <span
              class="py-0.5 px-2 grow hover:bg-[#304368b8]"
              @click="dialogAddEndpoint"
            >
              {{ $t("server-add-label") }}
            </span>
          </div>
        </div>
      </div>
    </div>
    <div class="flex gap-4 mt-6 text-2xl">
      <button
        class="font-main w-[160px] h-[56px] bg-[url('/classic/btn-blue.png')] state-bg shadow shadow-md shadow-black rounded-md uppercase"
        :disabled="store.authLoading"
        @click="doLogin"
      >
        {{ $t("login-button") }}
      </button>
      <button
        class="font-main w-[160px] h-[56px] bg-[url('/classic/btn-blue.png')] state-bg shadow shadow-md shadow-black rounded-md uppercase"
        :disabled="store.authLoading"
        @click="doRegister"
      >
        {{ $t("register-button") }}
      </button>
    </div>
    <label
      class="flex gap-2 items-center hover:brightness-150 mt-2"
      :class="store.authLoading ? 'disabled' : 'cursor-pointer'"
      @click="
        store.authLoading ? null : (storeMut.rememberMe = !storeMut.rememberMe)
      "
    >
      <img
        src="/classic/checkbox.png"
        draggable="false"
        class="h-[12px] w-[11px] object-none"
        :class="storeMut.rememberMe ? 'object-top' : 'object-bottom'"
      />
      <span class="text-sm">{{ $t("remember-me-label") }}</span>
    </label>
    <div
      class="grow bg-[#00000099] border-[1px] border-white/20 w-full rounded-sm m-2 p-[6px] text-[14px] leading-[14px] h-0 w-[426px] max-w-[426px]"
    >
      <div class="overflow-auto scrollbar h-full">
        <div v-for="log in store.log" style="overflow-anchor: none">
          <div :class="log.level">{{ log.message }}</div>
        </div>
        <div style="overflow-anchor: auto; height: 1px"></div>
      </div>
    </div>
  </div>
</template>
