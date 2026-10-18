import { onMounted, ref } from "vue";
import { createRule, deleteRule, listRules, updateRule } from "../lib/api";
import type { Rule, RuleInput } from "../lib/types";
import { useAttempt } from "./useAttempt";

export function useRules() {
  const rules = ref<Rule[]>([]);
  const { error, attempt } = useAttempt();

  onMounted(() =>
    attempt(async () => {
      rules.value = await listRules();
    }),
  );

  const create = (input: RuleInput) =>
    attempt(async () => {
      rules.value = [...rules.value, await createRule(input)];
    });

  const update = (rule: Rule) =>
    attempt(async () => {
      const saved = await updateRule(rule);
      rules.value = rules.value.map((existing) => (existing.id === saved.id ? saved : existing));
    });

  const remove = (id: string) =>
    attempt(async () => {
      await deleteRule(id);
      rules.value = rules.value.filter((rule) => rule.id !== id);
    });

  return { rules, error, create, update, remove };
}
