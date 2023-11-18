<script setup>
import { computed } from "@vue/reactivity";
import { useFluent } from "fluent-vue";

import { closeDropdown } from "../common";
import { availableLocales } from "../fluent";
import {
  storeMut,
  store,
  setCurrentEndpoint,
  doRegister,
  doLogin,
  dialogAddEndpoint,
  dialogEditEndpoint,
} from "../store";

const { $t } = useFluent();

const localeFlag = computed(() => `./src/icons/${props.locale}.svg`);

function onLocaleChange(locale) {
  document.activeElement.blur();
  storeMut.locale = locale;
}

setCurrentEndpoint(store.currentEndpoint);

function isCurrentEndpoint(endpoint) {
  return Object.entries(endpoint).every(
    ([k, v]) => v === store.currentEndpoint[k]
  );
}
</script>

<template>
  <div class="row-span-2 flex flex-col gap-3">
    <div class="mhf-card !px-5 flex gap-2">
      <div class="dropdown dropdown-origin dropdown-glass" @click.stop>
        <label
          tabindex="0"
          class="btn btn-sm btn-primary"
          :class="{ 'btn-disabled': store.authLoading }"
          @click.stop
        >
          <span>{{ store.currentEndpoint.name }}</span>
        </label>
        <div
          tabindex="0"
          class="dropdown-content z-[1] menu shadow shadow-black rounded-md w-max p-0 grid grid-cols-[1fr_auto] p-1 gap-x-0 overflow-auto scrollbar max-h-[440px]"
        >
          <template v-if="store.remoteEndpoints.length">
            <ul class="menu p-0">
              <li
                v-for="endpoint in store.remoteEndpoints"
                :key="endpoint.name"
                :class="{ active: isCurrentEndpoint(endpoint) }"
                @click="closeDropdown(() => setCurrentEndpoint(endpoint))"
              >
                <a>{{ endpoint.name }}</a>
              </li>
            </ul>
            <ul class="menu p-0">
              <li
                v-for="(_, i) in store.remoteEndpoints"
                @click="closeDropdown(() => dialogEditEndpoint(i, true))"
              >
                <a class="px-2">⚙</a>
              </li>
            </ul>
            <hr class="col-span-2 m-0" />
          </template>
          <template v-if="store.endpoints.length">
            <ul class="menu p-0">
              <li
                v-for="endpoint in store.endpoints"
                :key="endpoint.name"
                :class="{ active: isCurrentEndpoint(endpoint) }"
                @click="closeDropdown(() => setCurrentEndpoint(endpoint))"
              >
                <a>{{ endpoint.name }}</a>
              </li>
            </ul>
            <ul class="menu p-0">
              <li
                v-for="(_, i) in store.endpoints"
                @click="closeDropdown(() => dialogEditEndpoint(i, false))"
              >
                <a class="px-2">⚙</a>
              </li>
            </ul>
            <hr class="col-span-2 m-0" />
          </template>
          <ul class="menu col-span-2 p-0">
            <li @click="closeDropdown(dialogAddEndpoint)">
              <a>{{ $t("server-add-label") }}</a>
            </li>
          </ul>
        </div>
      </div>
      <div class="grow flex justify-end">
        <div class="dropdown dropdown-origin dropdown-glass dropdown-end">
          <label tabindex="0" class="btn btn-sm btn-primary">
            <img
              :src="`/flags/${storeMut.locale}.svg`"
              draggable="false"
              class="h-[12px]"
            />
            <div>{{ storeMut.locale.toUpperCase() }}</div>
          </label>
          <ul
            tabindex="0"
            class="dropdown-content z-[1] menu shadow shadow-black rounded-md w-max"
          >
            <li
              v-for="l in availableLocales"
              :key="l"
              :class="{ active: l === storeMut.locale }"
              @click="onLocaleChange(l)"
            >
              <a>
                <img
                  :src="`/flags/${l}.svg`"
                  draggable="false"
                  class="h-[12px]"
                />
                {{ l.toUpperCase() }}
              </a>
            </li>
          </ul>
        </div>
      </div>
    </div>
    <div class="mhf-card !py-3 !px-10 flex flex-col gap-2">
      <input
        v-model="storeMut.username"
        :disabled="store.authLoading"
        class="input input-sm input-primary"
        type="text"
        spellcheck="false"
        :placeholder="$t('username-label')"
      />
      <input
        v-model="storeMut.password"
        :disabled="store.authLoading"
        class="input input-sm input-primary"
        type="password"
        :placeholder="$t('password-label')"
      />
      <div class="flex flex-col">
        <label class="label cursor-pointer">
          <input
            v-model="storeMut.rememberMe"
            :disabled="store.authLoading"
            type="checkbox"
            class="checkbox checkbox-info checkbox-sm"
          />
          <span class="label-text" :class="{ disabled: store.authLoading }">
            {{ $t("remember-me-label") }}
          </span>
        </label>
      </div>
      <div class="flex gap-2 justify-center">
        <button
          class="btn btn-sm btn-primary"
          :disabled="store.authLoading"
          @click="doLogin"
        >
          {{ $t("login-button") }}
        </button>
        <button
          class="btn btn-sm btn-primary"
          :disabled="store.authLoading"
          @click="doRegister"
        >
          {{ $t("register-button") }}
        </button>
      </div>
    </div>
  </div>
</template>
