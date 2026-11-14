import { computed, nextTick, ref, watch, type Ref } from 'vue';
import type { Clause, CompanySettings } from '../types/company';

export function useCompanyClauses(settings: Ref<CompanySettings>, editing: (value: boolean) => void) {
  const editor = ref<Clause>();
  const isNew = ref(false);
  const error = ref('');
  const removed = ref<{ clause: Clause; index: number }>();
  const activeCount = computed(() => settings.value.clauses.filter(clause => clause.entry || clause.exit).length);
  let returnFocus: HTMLElement | null = null;

  watch(() => settings.value, () => { removed.value = undefined; });

  function edit(clause?: Clause) {
    returnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    editor.value = clause ? { ...clause } : {
      id: crypto.randomUUID(), title: '', text: '', entry: true, exit: false,
    };
    isNew.value = !clause;
    error.value = '';
    editing(true);
    void nextTick(() => document.getElementById('clause-title')?.focus());
  }

  function cancel() {
    editor.value = undefined;
    error.value = '';
    editing(false);
    void nextTick(() => returnFocus?.focus());
  }

  function apply() {
    if (!editor.value) return;
    const clause = { ...editor.value, title: editor.value.title.trim(), text: editor.value.text.trim() };
    if (!clause.title || !clause.text) {
      error.value = 'Completa el título y el texto de la cláusula.';
      void nextTick(() => document.getElementById(!clause.title ? 'clause-title' : 'clause-text')?.focus());
      return;
    }
    if (isNew.value) settings.value.clauses.push(clause);
    else {
      const index = settings.value.clauses.findIndex(item => item.id === clause.id);
      if (index >= 0) settings.value.clauses[index] = clause;
    }
    cancel();
  }

  function move(index: number, offset: number) {
    const destination = index + offset;
    if (index < 0 || index >= settings.value.clauses.length || destination < 0 || destination >= settings.value.clauses.length) return;
    const clause = settings.value.clauses.splice(index, 1)[0]!;
    settings.value.clauses.splice(destination, 0, clause);
    void nextTick(() => {
      const action = document.getElementById(`clause-${clause.id}-${offset < 0 ? 'up' : 'down'}`) as HTMLButtonElement | null;
      if (action && !action.disabled) action.focus();
      else document.getElementById(`clause-${clause.id}`)?.focus();
    });
  }

  function remove(index: number) {
    const clause = settings.value.clauses[index];
    if (!clause) return;
    removed.value = { clause: { ...clause }, index };
    settings.value.clauses.splice(index, 1);
    const next = settings.value.clauses[Math.min(index, settings.value.clauses.length - 1)];
    void nextTick(() => document.getElementById(next ? `clause-${next.id}` : 'clause-add')?.focus());
  }

  function undo() {
    if (!removed.value) return;
    const { clause, index } = removed.value;
    if (!settings.value.clauses.some(item => item.id === clause.id)) {
      settings.value.clauses.splice(Math.min(index, settings.value.clauses.length), 0, clause);
    }
    removed.value = undefined;
    void nextTick(() => document.getElementById(`clause-${clause.id}`)?.focus());
  }

  return { editor, isNew, error, removed, activeCount, edit, cancel, apply, move, remove, undo };
}
