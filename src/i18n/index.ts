import { ref } from "vue";
import { getLanguage } from "../lib/api";
import { onLanguageChanged } from "../lib/events";
import type { Language, LanguageInfo } from "../lib/types";
import en, { type Messages } from "./en";
import pt from "./pt";

const dictionaries: Record<Language, Messages> = { en, pt };

/** Shared by every component of the window; changing it re-renders the texts. */
const info = ref<LanguageInfo>({ language: "en", locale: "en-US" });

/** Dotted paths to every text, e.g. "rules.form.save". */
type Leaves<T, Prefix extends string = ""> = {
  [K in keyof T & string]: T[K] extends string
    ? `${Prefix}${K}`
    : Leaves<T[K], `${Prefix}${K}.`>;
}[keyof T & string];

export type MessageKey = Leaves<Messages>;

/** Translates a key, replacing `{name}` placeholders with `params`. */
export function t(key: MessageKey, params: Record<string, string | number> = {}): string {
  const text = key
    .split(".")
    .reduce<unknown>(
      (node, part) => (node as Record<string, unknown>)[part],
      dictionaries[info.value.language],
    ) as string;
  return text.replace(/\{(\w+)\}/g, (match, name: string) =>
    name in params ? String(params[name]) : match,
  );
}

/** BCP 47 tag for dates and numbers, following the language. */
export const locale = () => info.value.locale;

function apply(next: LanguageInfo) {
  info.value = next;
  document.documentElement.lang = next.locale;
}

/** Loads the language from the core and follows later changes. Call before mounting. */
export async function initI18n(): Promise<void> {
  try {
    apply(await getLanguage());
  } catch (error) {
    console.error("get_language failed", error);
  }
  await onLanguageChanged(apply);
}
