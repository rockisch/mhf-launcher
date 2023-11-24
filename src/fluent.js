import { FluentBundle, FluentResource } from "@fluent/bundle";
import { createFluentVue } from "fluent-vue";

import enMessages from "../locale/en.ftl?raw";
import jpMessages from "../locale/jp.ftl?raw";

const enBundle = new FluentBundle("en");
enBundle.addResource(new FluentResource(enMessages));

const jpBundle = new FluentBundle("jp");
jpBundle.addResource(new FluentResource(jpMessages));

const bundlesMap = {
  en: enBundle,
  jp: jpBundle,
};
export const availableLocales = Object.keys(bundlesMap);

let bundles = [];
export const fluentVue = createFluentVue({
  bundles,
});

export function setFluentLocale(locale) {
  bundles = [bundlesMap[locale], enBundle];
  fluentVue.bundles = bundles;
}

export function getMessage(message, args) {
  for (const bundle of bundles) {
    const msg = bundle.getMessage(message);
    if (!msg || !msg.value) continue;
    try {
      return bundle.formatPattern(msg.value, args);
    } catch (e) {}
  }
  return message;
}
