import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import zhCN from "./locales/zh-CN";
import en from "./locales/en";

export type Locale = "zh-CN" | "en";
// Map all leaf values to `string` so any locale with the same key structure is assignable
type ToStringLeaves<T> = {
  [K in keyof T]: T[K] extends object ? ToStringLeaves<T[K]> : string;
};
export type Messages = ToStringLeaves<typeof zhCN>;

const locales: Record<Locale, Messages> = { "zh-CN": zhCN, en };

const currentLocale = ref<Locale>("zh-CN");
const messages = computed(() => locales[currentLocale.value]);

export function t(key: string): string {
  const keys = key.split(".");
  let result: any = messages.value;
  for (const k of keys) {
    result = result?.[k];
  }
  return (result as string) ?? key;
}

export function getLocale() {
  return currentLocale;
}

export async function loadLocale() {
  try {
    const locale = await invoke<string>("get_locale");
    if (locale === "en" || locale === "zh-CN") {
      currentLocale.value = locale;
    }
  } catch {
    // default to zh-CN
  }
}

export async function setLocale(locale: Locale) {
  try {
    await invoke("set_locale", { locale });
    currentLocale.value = locale;
  } catch (e) {
    console.warn("set_locale failed:", e);
  }
}

export function getAvailableLocales(): { code: Locale; name: string }[] {
  return [
    { code: "zh-CN", name: "简体中文" },
    { code: "en", name: "English" },
  ];
}
