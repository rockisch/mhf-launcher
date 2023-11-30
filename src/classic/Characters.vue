<script setup>
import { ref, watch } from "vue";
import { computed } from "@vue/reactivity";

import Character from "./Character.vue";
import { LOGIN_PAGE, copyCid, openPicker } from "../common";
import {
  store,
  storeMut,
  doCreateCharacter,
  dialogDeleteCharacter,
  doExportCharacter,
  doSelectCharacter,
} from "../store";

const rootRef = ref(null);
const lastCharIndex = store.characters.findIndex(
  (c) => c.id === store.lastCharId
);
const characterIndex = ref(lastCharIndex >= 0 ? lastCharIndex : 0);
const animationClass = "character-animation";
const animationClassReverse = "character-animation-reverse";
let characterTimeout = null;
function clearAnimationClass() {
  for (const el of rootRef.value.querySelectorAll(".character")) {
    el.classList.remove(animationClass);
    el.classList.remove(animationClassReverse);
  }
}
function addAnimationClass(aclass) {
  for (const el of rootRef.value.querySelectorAll(".character")) {
    el.classList.add(aclass);
  }
}
watch(characterIndex, (newIndex, oldIndex) => {
  clearTimeout(characterTimeout);
  clearAnimationClass();
  const aclass = newIndex > oldIndex ? animationClass : animationClassReverse;
  setTimeout(() => {
    addAnimationClass(aclass);
    characterTimeout = setTimeout(clearAnimationClass, 300);
  }, 0);
});

watch(
  () => store.characters,
  () => {
    if (store.characters.length <= characterIndex.value) {
      characterIndex.value = 0;
    }
  }
);
const character = computed(() => store.characters[characterIndex.value]);
const nextCharacter = computed(
  () => store.characters[characterIndex.value + 1]
);
const nextNextCharacter = computed(
  () => store.characters[characterIndex.value + 2]
);
const prevCharacter = computed(
  () => store.characters[characterIndex.value - 1]
);
const prevPrevCharacter = computed(
  () => store.characters[characterIndex.value - 2]
);

const characterSettingsPicker = ref(false);
function openCharacterSettingsPicker() {
  openPicker(characterSettingsPicker);
}
</script>

<template>
  <div class="h-full w-full" ref="rootRef">
    <div class="flex flex-col items-center mt-4 mr-[-12px]">
      <div class="h-[39px] w-[132px] z-[10]">
        <img
          v-if="prevCharacter"
          src="/classic/btn-scroll-up.png"
          class="state-img h-[39px] w-[132px]"
          draggable="false"
          @click="characterIndex--"
        />
        <img
          v-else
          src="/classic/btn-scroll-up-disabled.png"
          draggable="false"
        />
      </div>
      <div class="absolute" v-if="prevPrevCharacter">
        <div class="relative z-[3] top-[-20px]">
          <Character
            class="character character-2"
            :character="prevPrevCharacter"
          ></Character>
        </div>
      </div>
      <div class="absolute" v-if="prevCharacter">
        <div class="relative z-[4]">
          <Character
            class="character character-1"
            :character="prevCharacter"
          ></Character>
        </div>
      </div>
      <Character
        class="character z-[5]"
        :character="character"
        :selectable="true"
      ></Character>
      <div class="absolute" v-if="nextCharacter">
        <div class="relative z-[4] top-[80px]">
          <Character
            class="character character-1"
            :character="nextCharacter"
          ></Character>
        </div>
      </div>
      <div class="absolute" v-if="nextNextCharacter">
        <div class="relative z-[3] top-[100px]">
          <Character
            class="character character-2"
            :character="nextNextCharacter"
          ></Character>
        </div>
      </div>
      <div class="h-[39px] w-[132px] z-[10]">
        <img
          v-if="nextCharacter"
          src="/classic/btn-scroll-down.png"
          class="state-img h-[39px] w-[132px]"
          draggable="false"
          @click="characterIndex++"
        />
        <img
          v-else
          src="/classic/btn-scroll-down-disabled.png"
          draggable="false"
        />
      </div>
    </div>
    <div class="flex justify-center mt-5 gap-3 pl-4">
      <button
        class="bg-[url('/classic/btn-generic.png')] h-[38px] w-[200px] state-bg"
        style="background-size: 200px 76px"
        @click="doCreateCharacter"
      >
        <span class="relative left-[-2px] top-[-5px] text-md">
          {{ $t("create-character-label") }}
        </span>
      </button>
      <div
        class="bg-[url('/classic/btn-generic.png')] h-[38px] w-[200px] state-bg"
        style="background-size: 200px 76px"
      >
        <button
          class="relative top-[-4px] flex items-center justify-center gap-2 w-full h-full py-1 pr-2 outline-0"
          @click="openCharacterSettingsPicker"
        >
          <span>
            {{ $t("options-character-label") }}
          </span>
          <span
            :class="characterSettingsPicker ? 'arrow-up' : 'arrow-down'"
          ></span>
        </button>
        <div v-if="characterSettingsPicker" class="absolute w-[192px]">
          <div
            class="relative top-[-5px] left-[2px] border-[1px] border-white/20 bg-[#000000f0] w-full rounded"
          >
            <button
              class="w-full px-2 py-0.5 hover:bg-[#304368b8]"
              @click="dialogDeleteCharacter(character)"
            >
              {{ $t("delete-character-label") }}
            </button>
            <button
              class="w-full px-2 py-0.5 hover:bg-[#304368b8]"
              @click="doExportCharacter(character.id)"
            >
              {{ $t("export-character-label") }}
            </button>
            <button
              class="w-full px-2 py-0.5 hover:bg-[#304368b8]"
              @click="copyCid(character.id)"
            >
              {{ $t("copy-cid-label") }}
            </button>
          </div>
        </div>
      </div>
    </div>
    <div class="flex justify-center mt-3">
      <button
        class="bg-[url('/classic/btn-blue.png')] h-[56px] w-[292px] shadow shadow-black rounded state-bg font-main text-3xl"
        @click="doSelectCharacter(character.id)"
      >
        {{ $t("start-game-label") }}
      </button>
    </div>
  </div>
</template>
