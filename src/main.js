import "./style.css";

import { createApp } from "vue";

import Main from "./Main.vue";
import { fluentVue } from "./fluent";

createApp(Main).use(fluentVue).mount("#app");
