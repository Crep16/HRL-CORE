import { onScopeDispose, ref } from 'vue';

export function useDraftLeaveGuard(actions: {
  save: () => Promise<boolean>;
  discard: () => boolean | void;
}) {
  const open = ref(false);
  const busy = ref(false);
  const error = ref('');
  let pending: Promise<boolean> | undefined;
  let resolve: ((allowed: boolean) => void) | undefined;

  function finish(allowed: boolean) {
    open.value = false;
    resolve?.(allowed);
    resolve = undefined;
    pending = undefined;
  }
  function request() {
    if (pending) return pending;
    error.value = '';
    open.value = true;
    pending = new Promise<boolean>(done => { resolve = done; });
    return pending;
  }
  async function save() {
    if (!open.value || busy.value) return;
    busy.value = true;
    error.value = '';
    try {
      if (await actions.save()) finish(true);
      else error.value = 'No se guardaron los cambios. Sigue editando para revisar los datos o vuelve a intentar guardar.';
    } catch {
      error.value = 'No se guardaron los cambios. Tu borrador se conserva; puedes volver a intentar guardar.';
    } finally { busy.value = false; }
  }
  function discard() {
    if (!open.value || busy.value) return;
    if (actions.discard() === false) {
      error.value = 'Confirma el resultado del guardado antes de descartar. El borrador y la solicitud se conservan.';
      return;
    }
    finish(true);
  }
  function keep() { if (!busy.value) finish(false); }
  onScopeDispose(() => finish(false));
  return { open, busy, error, request, save, discard, keep };
}
