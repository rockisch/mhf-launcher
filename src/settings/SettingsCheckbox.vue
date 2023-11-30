<script setup>
import SettingsItem from "./SettingsItem.vue";

const props = defineProps({
  name: String,
  modelValue: Boolean,
});
const emit = defineEmits(["update:modelValue"]);
</script>

<template>
  <div
    class="flex gap-1 items-center cursor-pointer"
    @click="emit('update:modelValue', !modelValue)"
  >
    <h2 class="text-xl">
      {{ name }}
    </h2>
    <div class="grow"></div>
    <label class="label cursor-pointer">
      <input
        :checked="modelValue"
        @click.stop
        @change="emit('update:modelValue', $event.target.checked)"
        type="checkbox"
        class="checkbox checkbox-info checkbox-sm"
      />
    </label>
  </div>
  <slot name="extended"></slot>
</template>
