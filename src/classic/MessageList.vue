<script setup>
import { computed } from "@vue/reactivity";

import { formatDate } from "../common";

const props = defineProps({
  title: String,
  important: Boolean,
  messages: Array,
});

const listClass = computed(() =>
  props.important ? "news-important" : "news-default"
);
</script>

<template>
  <div
    v-if="messages.length"
    class="col-span-3 w-full text-xl mb-[-5px]"
    :class="listClass"
  >
    <img
      :src="
        important
          ? '/classic/msg-line-important.png'
          : '/classic/msg-line-base.png'
      "
      draggable="false"
    />
    <div class="messages-header font-old relative bottom-[25px] left-[18px]">
      {{ title }}
    </div>
  </div>
  <template v-for="message in messages">
    <div class="ml-[18px]" :class="listClass">
      {{ formatDate(message.date) }}
    </div>
    <div
      class="cursor-pointer news-button"
      :class="listClass"
      @click="open(message.link)"
    >
      {{ message.message }}
    </div>
    <div>
      <img
        v-if="important"
        src="/classic/new.gif"
        class="mt-1.5"
        draggable="false"
      />
    </div>
  </template>
</template>
