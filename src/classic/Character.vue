<script setup>
import { useFluent } from "fluent-vue";

import { formatDate } from "../common";
import { doSelectCharacter } from "../store";

const { $t } = useFluent();

function getWeaponLabel(weapon) {
  switch (weapon) {
    case 0:
      return $t("greatsword-label");
    case 1:
      return $t("heavy-bowgun-label");
    case 2:
      return $t("hammer-label");
    case 3:
      return $t("lance-label");
    case 4:
      return $t("sword-and-shield-label");
    case 5:
      return $t("light-bowgun-label");
    case 6:
      return $t("dual-swords-label");
    case 7:
      return $t("longsword-label");
    case 8:
      return $t("hunting-horn-label");
    case 9:
      return $t("gunlance-label");
    case 10:
      return $t("bow-label");
    case 11:
      return $t("tonfa-label");
    case 12:
      return $t("switch-axe-label");
    case 13:
      return $t("magnetic-spike-label");
  }
}

const props = defineProps({
  character: Object,
  selectable: Boolean,
});
</script>

<template>
  <div
    class="text-black my-2 h-[143px] w-[520px] p-2"
    :style="{
      background: `url('/units/${character.weapon}.png')`,
    }"
  >
    <div
      class="w-full h-full flex flex-col items-center"
      :class="{ 'cursor-pointer': selectable }"
      @click="selectable ? doSelectCharacter(character.id) : null"
    >
      <div class="text-3xl mt-2 font-bold">{{ character.name }}</div>
      <div class="grow py-2 px-4 w-full h-full flex">
        <div class="flex-1 flex gap-2">
          <img
            :src="`/weapons/${character.weapon}.png`"
            class="h-[48px] m-2"
            draggable="false"
          />
          <div
            class="grow flex flex-col items-center leading-4 justify-center mr-7"
          >
            <div class="font-bold">{{ $t("weapon-label") }}</div>
            <div class="text-xl font-bold">
              {{ getWeaponLabel(character.weapon) }}
            </div>
          </div>
        </div>
        <div class="flex-1 flex flex-col text-lg leading-[1] mt-1">
          <div class="flex gap-4">
            <span>HR{{ character.hr }}</span>
            <span>GR{{ character.hr }}</span>
            <span class="font-mono">
              <span v-if="character.isFemale">♀</span>
              <span v-else>♂</span>
            </span>
          </div>
          <div>ID:{{ character.id }}</div>
          <div>
            {{
              `${$t("last-online-label")}:${formatDate(character.lastLogin)}`
            }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
