<script setup>
import {
  CHECKING_PATCHER,
  DOWNLOADING_PATCHER,
  PATCHING_PATCHER,
} from "../common";
import { store, cancelPatcher } from "../store";
</script>

<template>
  <div class="mhf-card flex flex-col gap-3 items-center">
    <span v-if="store.patcher.state === CHECKING_PATCHER">
      {{ $t("patcher-checking") }}
    </span>
    <span v-else-if="store.patcher.state === DOWNLOADING_PATCHER">
      {{
        $t("patcher-progress", {
          current: store.patcher.current,
          total: store.patcher.total,
        })
      }}
    </span>
    <span v-else>
      {{ $t("patcher-patching") }}
    </span>
    <progress
      class="progress"
      :max="store.patcher.current"
      :value="store.patcher.total"
    ></progress>
    <button
      class="btn btn-sm btn-primary px-8"
      @click="cancelPatcher"
      :disabled="store.patcher.state === PATCHING_PATCHER"
    >
      {{ $t("cancel-button") }}
    </button>
  </div>
</template>
