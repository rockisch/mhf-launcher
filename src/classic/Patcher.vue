<script setup>
import { ref } from "vue";

import { patcherPercentage } from "../store";
import { cancelPatcher } from "../store";
import { store } from "../store";
import { PATCHING_PATCHER } from "../common";

const frame = ref(0);
let count = 0;
setInterval(() => {
  count++;
  if (count >= 8) {
    count = 0;
  }
  switch (count) {
    case 0:
      frame.value = 0;
      break;
    case 1:
      frame.value = 1;
      break;
    case 4:
      frame.value = 2;
      break;
    case 5:
      frame.value = 3;
      break;
  }
}, 205);
</script>

<template>
  <div class="flex flex-col items-center">
    <div class="mt-10">{{ $t("updating-label") }}</div>
    <img
      src="/classic/progress.png"
      class="h-[91px] w-[85px] object-none"
      :style="{ 'object-position': `0 -${frame * 91}px` }"
    />
    <div>
      <img src="/classic/bar_frame.png" class="mt-10" />
      <img
        src="/classic/bar.jpg"
        class="object-left h-[6px] w-[302px] relative bottom-[11px] left-[8px]"
      />
      <div
        class="relative h-[6px] bottom-[17px] right-[8px] float-right bg-black"
        :style="{ width: `${302 - 302 * patcherPercentage}px` }"
      ></div>
    </div>
    <button
      class="box-text box-btn mt-[13px] mb-[12px]"
      :disabled="store.patcher.state === PATCHING_PATCHER"
      @click="cancelPatcher"
    >
      {{ $t("cancel-button") }}
    </button>
  </div>
</template>
