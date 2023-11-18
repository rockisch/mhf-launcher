<script setup>
import {
  store,
  doCreateCharacter,
  doSelectCharacter,
  doExportCharacter,
  dialogDeleteCharacter,
} from "../store";
import { LOGIN_PAGE, closeDropdown, formatDate } from "../common";
import { storeMut } from "../store";

function characterWeaponIcon(weapon) {
  return `/weapons/${weapon}.png`;
}
</script>

<template>
  <div class="mhf-card row-span-2 overflow-hidden">
    <div class="flex flex-col gap-2 h-full">
      <div class="flex flex-row-reverse justify-between gap-2">
        <div @click="storeMut.page = LOGIN_PAGE" class="btn btn-sm btn-primary">
          {{ $t("logout-button") }}
        </div>
        <div
          class="btn btn-sm btn-primary"
          v-if="store.characters.length > 0"
          @click="doCreateCharacter"
        >
          {{ $t("create-character-label") }}
        </div>
      </div>
      <div
        class="grid grid-cols-[1fr_auto] gap-2 overflow-auto scrollbar mr-[-4px] pr-[4px]"
      >
        <template v-for="character in store.characters">
          <button
            class="btn btn-primary h-[110px] grid grid-cols-9 grow gap-1 gap-x-4 px-3 py-3 items-end justify-items-start"
            :disabled="store.characterLoading"
            @click="doSelectCharacter(character.id)"
          >
            <img
              class="col-span-3 row-span-4 w-full h-full object-contain pt-0"
              draggable="false"
              :src="characterWeaponIcon(character.weapon)"
            />
            <div class="col-span-6 text-lg self-center">
              {{ character.name }}
            </div>
            <div class="col-span-6">
              {{ $t("character-gender-label") }}:
              <span v-if="character.isFemale">
                {{ $t("character-gender-female") }}
              </span>
              <span v-else>{{ $t("character-gender-male") }}</span>
            </div>
            <span class="col-span-6 flex gap-3 justify-end">
              <span>HR{{ character.hr }}</span>
              <span>GR{{ character.gr }}</span>
            </span>
            <div class="col-span-6">
              {{ $t("last-online-label") }}:
              <span>{{ formatDate(character.lastLogin) }}</span>
            </div>
          </button>
          <div class="dropdown dropdown-end">
            <label tabindex="0" class="btn btn-sm btn-primary">...</label>
            <ul
              tabindex="0"
              class="dropdown-content bg-[#111] z-[1] menu shadow shadow-black rounded-md w-max"
            >
              <li
                @click="closeDropdown(() => dialogDeleteCharacter(character))"
              >
                <a>{{ $t("delete-character-label") }}</a>
              </li>
              <li @click="closeDropdown(() => doExportCharacter(character.id))">
                <a>{{ $t("export-character-label") }}</a>
              </li>
            </ul>
          </div>
        </template>
        <div
          class="btn btn-primary h-[120px] col-span-2 text-white text-xl"
          v-if="store.characters.length === 0"
          @click="doCreateCharacter"
        >
          {{ $t("create-character-label") }}
        </div>
      </div>
    </div>
  </div>
</template>
